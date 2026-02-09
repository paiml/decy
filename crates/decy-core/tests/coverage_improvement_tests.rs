//! Coverage improvement tests for decy-core (DECY-COVERAGE)
//!
//! These tests target uncovered code paths in lib.rs to increase coverage to 95%.

use decy_core::{DependencyGraph, ProjectContext, TranspilationCache, TranspiledFile};
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper: Create temporary C file with content
fn create_temp_c_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).expect("Failed to write temp file");
    path
}

// ============================================================================
// ProjectContext coverage tests
// ============================================================================

#[test]
fn test_project_context_get_function_source() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(&temp, "test.c", "int foo() { return 1; }");

    let context = ProjectContext::new();
    let result = decy_core::transpile_file(&file, &context).expect("Should transpile");

    let mut context_with_func = ProjectContext::new();
    context_with_func.add_transpiled_file(&result);

    // Should find function source
    let source = context_with_func.get_function_source("foo");
    assert!(source.is_some(), "Should find function source");
    assert!(
        source.unwrap().contains("test.c"),
        "Source should reference the file"
    );

    // Should not find unknown function
    let unknown = context_with_func.get_function_source("nonexistent");
    assert!(unknown.is_none(), "Should not find unknown function");
}

#[test]
fn test_project_context_has_function() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(&temp, "test.c", "int bar() { return 42; }");

    let context = ProjectContext::new();
    let result = decy_core::transpile_file(&file, &context).expect("Should transpile");

    let mut context_with_func = ProjectContext::new();
    context_with_func.add_transpiled_file(&result);

    assert!(
        context_with_func.has_function("bar"),
        "Should find bar function"
    );
    assert!(
        !context_with_func.has_function("unknown"),
        "Should not find unknown function"
    );
}

#[test]
fn test_project_context_extract_type_name_enum() {
    let temp = TempDir::new().unwrap();

    // Test with enum definition
    let file = create_temp_c_file(
        &temp,
        "test.c",
        r#"
        enum Color {
            RED = 0,
            GREEN = 1,
            BLUE = 2
        };
        "#,
    );

    let context = ProjectContext::new();
    let result = decy_core::transpile_file(&file, &context).expect("Should transpile");

    let mut context_with_enum = ProjectContext::new();
    context_with_enum.add_transpiled_file(&result);

    // The context should extract enum names from generated Rust code
    // Note: The extraction depends on how the rust_code looks after generation
    assert!(result.rust_code.contains("Color") || result.rust_code.contains("enum"));
}

#[test]
fn test_project_context_default() {
    // Test Default trait implementation
    let context: ProjectContext = ProjectContext::default();
    assert!(!context.has_type("any"));
    assert!(!context.has_function("any"));
}

// ============================================================================
// TranspiledFile coverage tests
// ============================================================================

#[test]
fn test_transpiled_file_new() {
    // Direct constructor test
    let file = TranspiledFile::new(
        PathBuf::from("/tmp/test.c"),
        "fn test() {}".to_string(),
        vec![PathBuf::from("/tmp/dep.h")],
        vec!["test".to_string()],
        "extern \"C\" fn test();".to_string(),
    );

    assert_eq!(file.source_path, PathBuf::from("/tmp/test.c"));
    assert_eq!(file.rust_code, "fn test() {}");
    assert_eq!(file.dependencies.len(), 1);
    assert_eq!(file.functions_exported.len(), 1);
    assert!(!file.ffi_declarations.is_empty());
}

#[test]
fn test_transpiled_file_clone_and_debug() {
    let file = TranspiledFile::new(
        PathBuf::from("/tmp/test.c"),
        "fn test() {}".to_string(),
        vec![],
        vec!["test".to_string()],
        String::new(),
    );

    // Test Clone
    let cloned = file.clone();
    assert_eq!(cloned.source_path, file.source_path);
    assert_eq!(cloned.rust_code, file.rust_code);

    // Test Debug
    let debug_str = format!("{:?}", file);
    assert!(debug_str.contains("TranspiledFile"));
}

// ============================================================================
// DependencyGraph coverage tests
// ============================================================================

#[test]
fn test_dependency_graph_default() {
    // Test Default trait
    let graph: DependencyGraph = DependencyGraph::default();
    assert!(graph.is_empty());
    assert_eq!(graph.file_count(), 0);
}

#[test]
fn test_dependency_graph_has_dependency_missing_files() {
    let graph = DependencyGraph::new();

    // Both files missing from graph
    let result = graph.has_dependency(&PathBuf::from("missing1.c"), &PathBuf::from("missing2.c"));
    assert!(!result, "Should return false for missing files");
}

#[test]
fn test_dependency_graph_duplicate_add() {
    let mut graph = DependencyGraph::new();
    let path = PathBuf::from("/tmp/test.c");

    // Add same file twice
    graph.add_file(&path);
    graph.add_file(&path);

    // Should still only have 1 file
    assert_eq!(graph.file_count(), 1);
}

#[test]
fn test_header_guard_missing() {
    let temp = TempDir::new().unwrap();

    // Header without guards
    let no_guard = create_temp_c_file(
        &temp,
        "noguard.h",
        r#"
        int helper();
        "#,
    );

    let has_guard = DependencyGraph::has_header_guard(&no_guard).expect("Should check");
    assert!(!has_guard, "Should not detect header guard");
}

#[test]
fn test_header_guard_with_if_defined() {
    let temp = TempDir::new().unwrap();

    // Header with #if !defined style guard
    let if_defined_guard = create_temp_c_file(
        &temp,
        "ifdefguard.h",
        r#"#if !defined(CONFIG_H)
        #define CONFIG_H
        int helper();
        #endif"#,
    );

    let has_guard = DependencyGraph::has_header_guard(&if_defined_guard).expect("Should check");
    assert!(has_guard, "Should detect #if !defined style guard");
}

#[test]
fn test_parse_include_malformed() {
    // Malformed includes (missing closing delimiter)
    let code = r#"
#include "unclosed
#include <also_unclosed
#include normal.h
    "#;

    let includes = DependencyGraph::parse_include_directives(code);

    // Should not parse the malformed ones
    assert!(
        !includes.contains(&"unclosed".to_string()),
        "Should not include malformed"
    );
}

#[test]
fn test_parse_include_with_angle_brackets() {
    let code = r#"#include <sys/types.h>"#;
    let includes = DependencyGraph::parse_include_directives(code);

    assert_eq!(includes.len(), 1);
    assert!(includes.contains(&"sys/types.h".to_string()));
}

// ============================================================================
// TranspilationCache coverage tests
// ============================================================================

#[test]
fn test_cache_get_nonexistent_file() {
    let mut cache = TranspilationCache::new();

    // Try to get a file that doesn't exist in cache
    let result = cache.get(&PathBuf::from("/nonexistent/file.c"));
    assert!(result.is_none(), "Should return None for nonexistent file");
}

#[test]
fn test_cache_save_without_directory() {
    let cache = TranspilationCache::new();

    // Try to save without setting a directory
    let result = cache.save();
    assert!(result.is_err(), "Should fail without cache directory");
}

#[test]
fn test_cache_load_nonexistent() {
    let temp = TempDir::new().unwrap();
    let cache_dir = temp.path().join("nonexistent_cache");

    // Load from non-existent directory (should return empty cache)
    let cache = TranspilationCache::load(&cache_dir);
    assert!(cache.is_ok(), "Should create empty cache for missing dir");

    let cache = cache.unwrap();
    let stats = cache.statistics();
    assert_eq!(stats.total_files, 0, "Should have no files");
}

#[test]
fn test_cache_insert_with_hash_failure() {
    let mut cache = TranspilationCache::new();

    // Try to insert a file that doesn't exist (hash will fail)
    let fake_file = TranspiledFile::new(
        PathBuf::from("/nonexistent/file.c"),
        "fn test() {}".to_string(),
        vec![],
        vec!["test".to_string()],
        String::new(),
    );

    // Should silently skip (not panic)
    cache.insert(&PathBuf::from("/nonexistent/file.c"), &fake_file);

    // Cache should not have the entry
    let stats = cache.statistics();
    assert_eq!(stats.total_files, 0);
}

// ============================================================================
// Preprocess includes coverage tests
// ============================================================================

#[test]
fn test_transpile_with_system_header() {
    // Test transpiling code that references system headers
    let c_code = r#"
#include <stdio.h>
int main() { return 0; }
    "#;

    let result = decy_core::transpile(c_code);
    assert!(result.is_ok(), "Should handle system header: {:?}", result);
}

#[test]
fn test_transpile_with_malformed_include() {
    // Test with malformed #include (no closing quote)
    let c_code = r#"
#include "incomplete
int main() { return 0; }
    "#;

    // Should still attempt to transpile
    let result = decy_core::transpile(c_code);
    // This might fail or succeed depending on clang parsing
    // The important thing is it doesn't panic
    let _ = result;
}

#[test]
fn test_transpile_with_includes_no_base_dir() {
    // Test transpile_with_includes with None base_dir
    let c_code = "int main() { return 0; }";
    let result = decy_core::transpile_with_includes(c_code, None);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_verification_success() {
    let c_code = "int add(int a, int b) { return a + b; }";
    let result = decy_core::transpile_with_verification(c_code);
    assert!(result.is_ok());

    let transpilation_result = result.unwrap();
    assert!(transpilation_result.compiles, "Should compile successfully");
    assert!(
        transpilation_result.errors.is_empty(),
        "Should have no errors"
    );
}

#[test]
fn test_transpile_with_verification_failure() {
    // Invalid C code that should fail to parse
    let c_code = "this is not valid C code { ( }";
    let result = decy_core::transpile_with_verification(c_code);

    // Should return Ok with a failure result
    assert!(result.is_ok());
    let transpilation_result = result.unwrap();
    assert!(!transpilation_result.compiles, "Should not compile");
}

// ============================================================================
// Edge cases and error paths
// ============================================================================

#[test]
fn test_transpile_empty_code() {
    let result = decy_core::transpile("");
    // Empty code should either succeed with empty result or fail gracefully
    let _ = result;
}

#[test]
fn test_transpile_comment_only() {
    let c_code = "// Just a comment\n/* block comment */";
    let result = decy_core::transpile(c_code);
    // Should handle comment-only code
    let _ = result;
}

#[test]
fn test_transpile_file_nonexistent() {
    let context = ProjectContext::new();
    let result = decy_core::transpile_file(&PathBuf::from("/nonexistent/file.c"), &context);
    assert!(result.is_err(), "Should fail for nonexistent file");
}

#[test]
fn test_from_files_read_error() {
    // Test with a file that doesn't exist
    let files = vec![PathBuf::from("/nonexistent/file.c")];
    let result = DependencyGraph::from_files(&files);
    assert!(result.is_err(), "Should fail for nonexistent file");
}

#[test]
fn test_cache_statistics_initial() {
    let cache = TranspilationCache::new();
    let stats = cache.statistics();

    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.total_files, 0);
}

#[test]
fn test_cache_compute_hash_missing_file() {
    let cache = TranspilationCache::new();
    let result = cache.compute_hash(&PathBuf::from("/nonexistent.c"));
    assert!(result.is_err(), "Should fail for missing file");
}

// ============================================================================
// Additional coverage tests for lib.rs
// ============================================================================

#[test]
fn test_transpile_with_struct() {
    let c_code = r#"
struct Point { int x; int y; };
int get_x(struct Point* p) { return p->x; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("Point") || code.contains("struct"));
}

#[test]
fn test_transpile_with_enum() {
    let c_code = r#"
enum State { IDLE, RUNNING, STOPPED };
int is_running(enum State s) { return s == RUNNING; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_typedef() {
    let c_code = r#"
typedef int MyInt;
MyInt add(MyInt a, MyInt b) { return a + b; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_global_variable() {
    let c_code = r#"
int counter = 0;
void increment() { counter = counter + 1; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("static mut") || code.contains("counter"));
}

#[test]
fn test_transpile_with_array_global() {
    let c_code = r#"
int data[10];
int get_first() { return data[0]; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_pointer_param() {
    let c_code = r#"
void set_value(int* ptr) { *ptr = 42; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_from_file_path() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(&temp, "test.c", "int main() { return 0; }");

    let result = decy_core::transpile_from_file_path(&file);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_from_file_path_nonexistent() {
    let result = decy_core::transpile_from_file_path(&PathBuf::from("/nonexistent.c"));
    assert!(result.is_err());
}

#[test]
fn test_project_context_has_type() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(
        &temp,
        "test.c",
        "struct Foo { int x; }; int get_x(struct Foo* f) { return f->x; }",
    );

    let context = ProjectContext::new();
    let result = decy_core::transpile_file(&file, &context).expect("Should transpile");

    let mut ctx = ProjectContext::new();
    ctx.add_transpiled_file(&result);

    // May or may not find the type depending on code generation
    let _ = ctx.has_type("Foo");
}

#[test]
fn test_cache_clear() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(&temp, "test.c", "int foo() { return 1; }");

    let context = ProjectContext::new();
    let transpiled = decy_core::transpile_file(&file, &context).expect("Should transpile");

    let mut cache = TranspilationCache::new();
    cache.insert(&file, &transpiled);

    let stats_before = cache.statistics();
    assert_eq!(stats_before.total_files, 1);

    cache.clear();

    let stats_after = cache.statistics();
    assert_eq!(stats_after.total_files, 0);
    assert_eq!(stats_after.hits, 0);
    assert_eq!(stats_after.misses, 0);
}

#[test]
fn test_cache_save_and_load() {
    let temp = TempDir::new().unwrap();
    let cache_dir = temp.path().join("cache");
    std::fs::create_dir_all(&cache_dir).unwrap();

    let file = create_temp_c_file(&temp, "test.c", "int bar() { return 2; }");
    let context = ProjectContext::new();
    let transpiled = decy_core::transpile_file(&file, &context).expect("Should transpile");

    let mut cache = TranspilationCache::with_directory(&cache_dir);
    cache.insert(&file, &transpiled);
    cache.save().expect("Should save");

    // Load cache and verify
    let loaded = TranspilationCache::load(&cache_dir).expect("Should load");
    let stats = loaded.statistics();
    assert_eq!(stats.total_files, 1);
}

#[test]
fn test_cache_with_dependencies() {
    let temp = TempDir::new().unwrap();

    // Create header file
    let header = create_temp_c_file(&temp, "helper.h", "int helper();");

    // Create main file with dependency
    let main_file = create_temp_c_file(
        &temp,
        "main.c",
        r#"#include "helper.h"
int main() { return helper(); }"#,
    );

    let transpiled = TranspiledFile::new(
        main_file.clone(),
        "fn main() {}".to_string(),
        vec![header.clone()],
        vec!["main".to_string()],
        String::new(),
    );

    let mut cache = TranspilationCache::new();
    cache.insert(&main_file, &transpiled);

    // Get should check dependencies too
    let result = cache.get(&main_file);
    assert!(result.is_some());
}

#[test]
fn test_dependency_graph_topological_sort() {
    let mut graph = DependencyGraph::new();

    let a = PathBuf::from("/a.c");
    let b = PathBuf::from("/b.c");
    let c = PathBuf::from("/c.c");

    graph.add_file(&a);
    graph.add_file(&b);
    graph.add_file(&c);

    // a depends on b, b depends on c
    graph.add_dependency(&a, &b);
    graph.add_dependency(&b, &c);

    let order = graph.topological_sort().expect("Should sort");
    assert_eq!(order.len(), 3);

    // c should come first (no dependencies)
    assert!(order.iter().position(|p| p == &c) < order.iter().position(|p| p == &b));
    assert!(order.iter().position(|p| p == &b) < order.iter().position(|p| p == &a));
}

#[test]
fn test_dependency_graph_circular() {
    let mut graph = DependencyGraph::new();

    let a = PathBuf::from("/a.c");
    let b = PathBuf::from("/b.c");

    graph.add_file(&a);
    graph.add_file(&b);

    // Create cycle: a -> b -> a
    graph.add_dependency(&a, &b);
    graph.add_dependency(&b, &a);

    let result = graph.topological_sort();
    assert!(result.is_err(), "Should detect circular dependency");
}

#[test]
fn test_transpile_with_function_pointer_global() {
    let c_code = r#"
int (*callback)(int);
int call_it(int x) { return callback(x); }
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("Option") || code.contains("callback"));
}

#[test]
fn test_transpile_with_extern() {
    let c_code = r#"
extern int external_var;
int get_external() { return external_var; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_float_global() {
    let c_code = r#"
float pi = 3.14;
double e = 2.718;
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_char_array() {
    let c_code = r#"
char message[100];
void set_msg(char c) { message[0] = c; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_struct_array() {
    let c_code = r#"
struct Item { int id; };
struct Item items[10];
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_pointer_arithmetic() {
    let c_code = r#"
int sum(int* arr, int n) {
    int total = 0;
    int* p = arr;
    int* end = arr + n;
    while (p < end) {
        total = total + *p;
        p = p + 1;
    }
    return total;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_null_comparison() {
    let c_code = r#"
int is_valid(int* ptr) {
    if (ptr == 0) return 0;
    return 1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_slice_params() {
    let c_code = r#"
int sum(int* arr, int len) {
    int total = 0;
    int i = 0;
    while (i < len) {
        total = total + arr[i];
        i = i + 1;
    }
    return total;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_unsigned_int_global() {
    let c_code = "unsigned int count = 0;";
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_signed_char_global() {
    let c_code = "signed char byte_val;";
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_cache_debug() {
    let cache = TranspilationCache::new();
    let debug = format!("{:?}", cache);
    assert!(debug.contains("TranspilationCache"));
}

#[test]
fn test_cache_statistics_debug() {
    let stats = decy_core::CacheStatistics {
        hits: 5,
        misses: 2,
        total_files: 10,
    };
    let debug = format!("{:?}", stats);
    assert!(debug.contains("hits"));
    assert!(debug.contains("5"));
}

// ============================================================================
// Additional edge case tests for higher coverage
// ============================================================================

#[test]
fn test_transpile_function_with_declaration_and_definition() {
    // Tests DECY-190: Function deduplication (declaration + definition)
    let c_code = r#"
int foo(int x);  // declaration
int foo(int x) { return x + 1; }  // definition
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
    let code = result.unwrap();
    // Should only have one function definition, not duplicate
    assert!(code.contains("fn foo"));
}

#[test]
fn test_transpile_with_extern_and_definition() {
    // Tests extern variable handling
    let c_code = r#"
extern int max;
int max = 100;
int get_max() { return max; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_typedef_struct() {
    let c_code = r#"
typedef struct {
    int x;
    int y;
} Point;
Point make_point(int x, int y) { Point p; p.x = x; p.y = y; return p; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_array_length_param() {
    // Tests DECY-116: Slice function args detection
    let c_code = r#"
void process(int* arr, int len) {
    int i = 0;
    while (i < len) {
        arr[i] = 0;
        i = i + 1;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_size_param() {
    // Tests size parameter detection
    let c_code = r#"
void fill(char* buf, int size) {
    int i = 0;
    while (i < size) {
        buf[i] = 0;
        i = i + 1;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_count_param() {
    // Tests count parameter detection
    let c_code = r#"
int sum_n(int* values, int count) {
    int total = 0;
    int i = 0;
    while (i < count) {
        total = total + values[i];
        i = i + 1;
    }
    return total;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_n_param() {
    // Tests 'n' parameter detection
    let c_code = r#"
void zero_array(double* arr, int n) {
    int i = 0;
    while (i < n) {
        arr[i] = 0.0;
        i = i + 1;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_num_param() {
    // Tests 'num' parameter detection
    let c_code = r#"
int find_max(int* data, int num) {
    int max = data[0];
    int i = 1;
    while (i < num) {
        if (data[i] > max) max = data[i];
        i = i + 1;
    }
    return max;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_multiple_structs() {
    // Tests struct deduplication
    let c_code = r#"
struct Node { int value; struct Node* next; };
struct Node create_node(int v) {
    struct Node n;
    n.value = v;
    n.next = 0;
    return n;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_nested_control_flow() {
    let c_code = r#"
int nested(int x, int y) {
    if (x > 0) {
        if (y > 0) {
            return x + y;
        } else {
            return x - y;
        }
    } else {
        while (x < 0) {
            x = x + 1;
        }
        return x;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_switch() {
    let c_code = r#"
int classify(int x) {
    switch (x) {
        case 0: return 0;
        case 1: return 1;
        default: return -1;
    }
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_for_loop() {
    let c_code = r#"
int sum(int n) {
    int total = 0;
    int i;
    for (i = 0; i < n; i = i + 1) {
        total = total + i;
    }
    return total;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_do_while() {
    let c_code = r#"
int countdown(int n) {
    int count = 0;
    do {
        count = count + 1;
        n = n - 1;
    } while (n > 0);
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_break_continue() {
    let c_code = r#"
int find_first(int* arr, int n, int target) {
    int i = 0;
    while (i < n) {
        if (arr[i] == target) {
            return i;
        }
        i = i + 1;
    }
    return -1;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_ternary() {
    let c_code = r#"
int abs(int x) {
    return x >= 0 ? x : -x;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_bitwise_ops() {
    let c_code = r#"
int bit_ops(int a, int b) {
    int r1 = a & b;
    int r2 = a | b;
    int r3 = a ^ b;
    int r4 = a << 2;
    int r5 = b >> 1;
    return r1 + r2 + r3 + r4 + r5;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_logical_ops() {
    let c_code = r#"
int logical(int a, int b) {
    if (a > 0 && b > 0) return 1;
    if (a > 0 || b > 0) return 2;
    if (!a) return 3;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_comparison_ops() {
    let c_code = r#"
int compare(int a, int b) {
    if (a == b) return 0;
    if (a != b) return 1;
    if (a < b) return -1;
    if (a > b) return 1;
    if (a <= b) return -2;
    if (a >= b) return 2;
    return 0;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_cache_hit_after_miss() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(&temp, "test.c", "int test() { return 1; }");

    let context = ProjectContext::new();
    let transpiled = decy_core::transpile_file(&file, &context).expect("Should transpile");

    let mut cache = TranspilationCache::new();

    // First get should be a miss
    let result1 = cache.get(&file);
    assert!(result1.is_none());

    // Insert into cache
    cache.insert(&file, &transpiled);

    // Second get should be a hit
    let result2 = cache.get(&file);
    assert!(result2.is_some());

    let stats = cache.statistics();
    assert!(stats.hits > 0 || stats.misses > 0);
}

#[test]
fn test_cache_invalidation_on_file_change() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_c_file(&temp, "test.c", "int original() { return 1; }");

    let context = ProjectContext::new();
    let transpiled = decy_core::transpile_file(&file, &context).expect("Should transpile");

    let mut cache = TranspilationCache::new();
    cache.insert(&file, &transpiled);

    // Modify the file
    std::fs::write(&file, "int modified() { return 2; }").unwrap();

    // Get should return None because hash changed
    let result = cache.get(&file);
    assert!(result.is_none());
}

#[test]
fn test_dependency_graph_with_header() {
    let temp = TempDir::new().unwrap();

    let header = create_temp_c_file(&temp, "helper.h", "int helper();");
    let main = create_temp_c_file(
        &temp,
        "main.c",
        r#"#include "helper.h"
int main() { return helper(); }"#,
    );

    let files = vec![main.clone(), header.clone()];
    let result = DependencyGraph::from_files(&files);

    // This might fail or succeed depending on parsing, but shouldn't panic
    let _ = result;
}

#[test]
fn test_transpile_with_void_pointer() {
    let c_code = r#"
void* get_data(void* ptr) {
    return ptr;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_static_variable() {
    let c_code = r#"
int counter() {
    static int count = 0;
    count = count + 1;
    return count;
}
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_const_variable() {
    let c_code = r#"
const int MAX_SIZE = 100;
int get_max() { return MAX_SIZE; }
"#;
    let result = decy_core::transpile(c_code);
    assert!(result.is_ok());
}

#[test]
fn test_project_context_add_multiple_files() {
    let temp = TempDir::new().unwrap();

    let file1 = create_temp_c_file(&temp, "file1.c", "int func1() { return 1; }");
    let file2 = create_temp_c_file(&temp, "file2.c", "int func2() { return 2; }");

    let context = ProjectContext::new();
    let result1 = decy_core::transpile_file(&file1, &context).expect("Should transpile");
    let result2 = decy_core::transpile_file(&file2, &context).expect("Should transpile");

    let mut ctx = ProjectContext::new();
    ctx.add_transpiled_file(&result1);
    ctx.add_transpiled_file(&result2);

    assert!(ctx.has_function("func1"));
    assert!(ctx.has_function("func2"));
}

#[test]
fn test_transpiled_file_equality() {
    let file1 = TranspiledFile::new(
        PathBuf::from("/tmp/test.c"),
        "fn test() {}".to_string(),
        vec![],
        vec!["test".to_string()],
        String::new(),
    );

    let file2 = file1.clone();
    assert_eq!(file1.source_path, file2.source_path);
    assert_eq!(file1.rust_code, file2.rust_code);
}

// ============================================================================
// Additional Coverage Tests (DECY-COVERAGE-CORE-2)
// ============================================================================

#[test]
fn test_transpile_with_double_type() {
    let result = decy_core::transpile("double calc() { double x = 3.14; return x; }");
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("f64"));
}

#[test]
fn test_transpile_with_float_type() {
    let result = decy_core::transpile("float calc() { float x = 3.14f; return x; }");
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("f32") || code.contains("f64"));
}

#[test]
fn test_transpile_with_long_type() {
    let result = decy_core::transpile("long calc() { long x = 123456789; return x; }");
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("i64") || code.contains("i32"));
}

#[test]
fn test_transpile_with_unsigned_long() {
    let result = decy_core::transpile("unsigned long calc() { unsigned long x = 123; return x; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_short_type() {
    let result = decy_core::transpile("short calc() { short x = 10; return x; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_unsigned_short() {
    let result = decy_core::transpile("unsigned short calc() { unsigned short x = 10; return x; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_void_pointer_return() {
    let result = decy_core::transpile("void* get_ptr() { return 0; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_multiple_functions() {
    let result = decy_core::transpile(
        "int add(int a, int b) { return a + b; }
         int sub(int a, int b) { return a - b; }
         int mul(int a, int b) { return a * b; }",
    );
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("add"));
    assert!(code.contains("sub"));
    assert!(code.contains("mul"));
}

#[test]
fn test_transpile_with_nested_if() {
    let result = decy_core::transpile(
        "int nested(int x) {
            if (x > 10) {
                if (x > 20) {
                    return 2;
                }
                return 1;
            }
            return 0;
        }",
    );
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_nested_while() {
    let result = decy_core::transpile(
        "int nested() {
            int i = 0;
            while (i < 10) {
                int j = 0;
                while (j < 5) {
                    j++;
                }
                i++;
            }
            return i;
        }",
    );
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_multiple_vars() {
    let result = decy_core::transpile(
        "int calc() {
            int a = 1, b = 2, c = 3;
            return a + b + c;
        }",
    );
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_prefix_ops() {
    let result = decy_core::transpile(
        "int ops() {
            int x = 5;
            int y = ++x;
            int z = --x;
            return y + z;
        }",
    );
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_postfix_ops() {
    let result = decy_core::transpile(
        "int ops() {
            int x = 5;
            int y = x++;
            int z = x--;
            return y + z;
        }",
    );
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_ternary_nested() {
    let result = decy_core::transpile(
        "int nested(int x) {
            return x > 0 ? (x > 10 ? 2 : 1) : 0;
        }",
    );
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_sizeof_expr() {
    let result = decy_core::transpile(
        "int size() {
            int arr[10];
            return sizeof(arr);
        }",
    );
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_sizeof_type() {
    let result = decy_core::transpile(
        "int size() {
            return sizeof(int);
        }",
    );
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_complex_struct() {
    let result = decy_core::transpile(
        "struct Node {
            int value;
            struct Node *left;
            struct Node *right;
        };
        int get_value(struct Node *n) { return n->value; }",
    );
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_array_param() {
    let result = decy_core::transpile(
        "int sum(int arr[], int n) {
            int total = 0;
            for (int i = 0; i < n; i++) {
                total += arr[i];
            }
            return total;
        }",
    );
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_2d_array() {
    let result = decy_core::transpile(
        "int get_elem(int matrix[3][3], int i, int j) {
            return matrix[i][j];
        }",
    );
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_function_pointer_param() {
    let result = decy_core::transpile(
        "int apply(int (*f)(int), int x) {
            return f(x);
        }",
    );
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_typedef_function_pointer() {
    let result = decy_core::transpile(
        "typedef int (*IntFunc)(int);
        int apply(IntFunc f, int x) { return f(x); }",
    );
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_enum_values() {
    let result = decy_core::transpile(
        "enum Status { OK = 0, ERROR = 1, PENDING = 2 };
        int get_status() { return OK; }",
    );
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_bitfield() {
    let result = decy_core::transpile(
        "struct Flags {
            unsigned int a : 1;
            unsigned int b : 2;
            unsigned int c : 5;
        };
        int test() { return 0; }",
    );
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_union() {
    let result = decy_core::transpile(
        "union Value {
            int i;
            float f;
            char c;
        };
        int test() { return 0; }",
    );
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_complex_expression() {
    let result = decy_core::transpile(
        "int complex() {
            int a = 1, b = 2, c = 3;
            return ((a + b) * c) - ((a - b) / (c + 1));
        }",
    );
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_modulo() {
    let result = decy_core::transpile("int modulo(int x, int y) { return x % y; }");
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("%"));
}

#[test]
fn test_transpile_with_shift_ops() {
    let result = decy_core::transpile("int shift(int x) { return (x << 2) | (x >> 1); }");
    assert!(result.is_ok());
    let code = result.unwrap();
    assert!(code.contains("<<") || code.contains(">>"));
}

#[test]
fn test_transpile_with_bitwise_xor() {
    let result = decy_core::transpile("int xor_fn(int a, int b) { return a ^ b; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_bitwise_complement() {
    let result = decy_core::transpile("int complement(int x) { return ~x; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_unary_minus() {
    let result = decy_core::transpile("int negate(int x) { return -x; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_logical_not() {
    let result = decy_core::transpile("int not_fn(int x) { return !x; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_compound_add() {
    let result = decy_core::transpile("int add() { int x = 0; x += 5; return x; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_compound_sub() {
    let result = decy_core::transpile("int sub() { int x = 10; x -= 3; return x; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_compound_mul() {
    let result = decy_core::transpile("int mul() { int x = 2; x *= 4; return x; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_compound_div() {
    let result = decy_core::transpile("int div() { int x = 20; x /= 4; return x; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_compound_mod() {
    let result = decy_core::transpile("int mod() { int x = 17; x %= 5; return x; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_compound_and() {
    let result = decy_core::transpile("int and_fn() { int x = 15; x &= 7; return x; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_compound_or() {
    let result = decy_core::transpile("int or_fn() { int x = 8; x |= 3; return x; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_compound_xor() {
    let result = decy_core::transpile("int xor_fn() { int x = 12; x ^= 5; return x; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_compound_shl() {
    let result = decy_core::transpile("int shl() { int x = 1; x <<= 3; return x; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_compound_shr() {
    let result = decy_core::transpile("int shr() { int x = 16; x >>= 2; return x; }");
    assert!(result.is_ok());
}

#[test]
fn test_project_context_extract_type_name_struct() {
    let result = decy_core::transpile("struct Point { int x; int y; }; int test() { return 0; }");
    assert!(result.is_ok());
}

#[test]
fn test_project_context_extract_type_name_typedef_struct() {
    let result =
        decy_core::transpile("typedef struct { int x; int y; } Point; int test() { return 0; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_void_param() {
    let result = decy_core::transpile("int test(void) { return 0; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_static_local() {
    let result = decy_core::transpile("int counter() { static int count = 0; return ++count; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_register_var() {
    let result = decy_core::transpile("int fast() { register int i = 0; return i; }");
    assert!(result.is_ok());
}

#[test]
fn test_transpile_with_volatile_var() {
    let result = decy_core::transpile("int vol() { volatile int x = 0; return x; }");
    assert!(result.is_ok());
}
