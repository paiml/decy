//! Dependency tracking and build order tests (DECY-048 RED phase)
//!
//! These tests define the API for tracking file dependencies and computing
//! the correct transpilation order based on #include relationships.
//!
//! Goal: Enable correct build ordering for multi-file C projects.

use decy_core::DependencyGraph;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper: Create temporary C file with content
fn create_temp_c_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).expect("Failed to write temp file");
    path
}

#[test]
fn test_dependency_graph_empty() {
    // Test: Empty dependency graph
    let graph = DependencyGraph::new();

    assert_eq!(graph.file_count(), 0, "Empty graph should have 0 files");
    assert!(graph.is_empty(), "Empty graph should report as empty");
}

#[test]
fn test_dependency_graph_add_file() {
    // Test: Add a single file to dependency graph
    let mut graph = DependencyGraph::new();
    let path = PathBuf::from("/tmp/test.c");

    graph.add_file(&path);

    assert_eq!(graph.file_count(), 1, "Should have 1 file");
    assert!(graph.contains_file(&path), "Should contain the added file");
}

#[test]
fn test_dependency_graph_add_dependency() {
    // Test: Add a dependency relationship between two files
    let mut graph = DependencyGraph::new();
    let main_c = PathBuf::from("/tmp/main.c");
    let utils_h = PathBuf::from("/tmp/utils.h");

    graph.add_file(&main_c);
    graph.add_file(&utils_h);
    graph.add_dependency(&main_c, &utils_h);

    assert!(
        graph.has_dependency(&main_c, &utils_h),
        "Should have dependency"
    );
    assert!(
        !graph.has_dependency(&utils_h, &main_c),
        "Dependency is directional"
    );
}

#[test]
fn test_build_dependency_graph_from_files() {
    // Test: Build dependency graph from actual C files with #include directives
    let temp = TempDir::new().unwrap();

    // utils.h - no dependencies
    let utils_h = create_temp_c_file(&temp, "utils.h", "int helper(int x);");

    // main.c - includes utils.h
    let main_c = create_temp_c_file(
        &temp,
        "main.c",
        r#"#include "utils.h"
        int main() { return helper(5); }"#,
    );

    let files = vec![main_c.clone(), utils_h.clone()];
    let graph = DependencyGraph::from_files(&files).expect("Should build graph");

    assert_eq!(graph.file_count(), 2, "Should have 2 files");
    assert!(
        graph.has_dependency(&main_c, &utils_h),
        "main.c should depend on utils.h"
    );
}

#[test]
fn test_topological_sort_simple() {
    // Test: Topological sort gives correct build order
    let mut graph = DependencyGraph::new();
    let main_c = PathBuf::from("main.c");
    let utils_h = PathBuf::from("utils.h");

    graph.add_file(&main_c);
    graph.add_file(&utils_h);
    graph.add_dependency(&main_c, &utils_h);

    let build_order = graph
        .topological_sort()
        .expect("Should compute build order");

    assert_eq!(build_order.len(), 2, "Should have 2 files in order");

    // utils.h should come before main.c (no dependencies before dependencies)
    let utils_pos = build_order.iter().position(|p| p == &utils_h).unwrap();
    let main_pos = build_order.iter().position(|p| p == &main_c).unwrap();
    assert!(
        utils_pos < main_pos,
        "utils.h should be transpiled before main.c"
    );
}

#[test]
fn test_topological_sort_complex() {
    // Test: Complex dependency chain with multiple levels
    // a.h -> b.h -> c.h
    let mut graph = DependencyGraph::new();
    let a = PathBuf::from("a.h");
    let b = PathBuf::from("b.h");
    let c = PathBuf::from("c.h");

    graph.add_file(&a);
    graph.add_file(&b);
    graph.add_file(&c);
    graph.add_dependency(&a, &b);
    graph.add_dependency(&b, &c);

    let build_order = graph
        .topological_sort()
        .expect("Should compute build order");

    // c should come first (no dependencies), then b, then a
    let c_pos = build_order.iter().position(|p| p == &c).unwrap();
    let b_pos = build_order.iter().position(|p| p == &b).unwrap();
    let a_pos = build_order.iter().position(|p| p == &a).unwrap();

    assert!(c_pos < b_pos, "c should come before b");
    assert!(b_pos < a_pos, "b should come before a");
}

#[test]
fn test_detect_circular_dependency() {
    // Test: Circular dependency detection (a -> b -> a)
    let mut graph = DependencyGraph::new();
    let a = PathBuf::from("a.h");
    let b = PathBuf::from("b.h");

    graph.add_file(&a);
    graph.add_file(&b);
    graph.add_dependency(&a, &b);
    graph.add_dependency(&b, &a);

    let result = graph.topological_sort();

    assert!(result.is_err(), "Should detect circular dependency");
    let error_msg = result.unwrap_err().to_string();
    let error_msg_lower = error_msg.to_lowercase();
    assert!(
        error_msg_lower.contains("circular") || error_msg_lower.contains("cycle"),
        "Error should mention circular dependency"
    );
}

#[test]
fn test_detect_self_dependency() {
    // Test: Self-dependency detection (a -> a)
    let mut graph = DependencyGraph::new();
    let a = PathBuf::from("a.h");

    graph.add_file(&a);
    graph.add_dependency(&a, &a);

    let result = graph.topological_sort();

    assert!(result.is_err(), "Should detect self-dependency as circular");
}

#[test]
fn test_header_guard_detection() {
    // Test: Detect header guards to prevent duplicate processing
    let temp = TempDir::new().unwrap();

    let header = create_temp_c_file(
        &temp,
        "config.h",
        r#"#ifndef CONFIG_H
        #define CONFIG_H
        int MAX_SIZE = 100;
        #endif"#,
    );

    let has_guard = DependencyGraph::has_header_guard(&header).expect("Should check guard");

    assert!(has_guard, "Should detect header guard");
}

#[test]
fn test_parse_include_directive() {
    // Test: Parse #include directive to extract filename
    let includes = [
        r#"#include "utils.h""#,
        r#"#include <stdio.h>"#,
        r#"  #include   "config.h"  "#,
    ];

    let parsed = DependencyGraph::parse_include_directives(&includes.join("\n"));

    assert_eq!(parsed.len(), 3, "Should find 3 includes");
    assert!(parsed.contains(&"utils.h".to_string()));
    assert!(parsed.contains(&"stdio.h".to_string()));
    assert!(parsed.contains(&"config.h".to_string()));
}

#[test]
fn test_build_order_integration() {
    // Integration test: Build correct order for realistic file structure
    let temp = TempDir::new().unwrap();

    // types.h - no dependencies
    let types_h = create_temp_c_file(&temp, "types.h", "typedef struct { int x; } Point;");

    // utils.h - depends on types.h
    let utils_h = create_temp_c_file(
        &temp,
        "utils.h",
        r#"#include "types.h"
        Point create_point(int x);"#,
    );

    // main.c - depends on utils.h
    let main_c = create_temp_c_file(
        &temp,
        "main.c",
        r#"#include "utils.h"
        int main() { return 0; }"#,
    );

    let files = vec![main_c.clone(), utils_h.clone(), types_h.clone()];
    let graph = DependencyGraph::from_files(&files).expect("Should build graph");
    let build_order = graph.topological_sort().expect("Should compute order");

    // Verify build order: types.h -> utils.h -> main.c
    let types_pos = build_order.iter().position(|p| p == &types_h).unwrap();
    let utils_pos = build_order.iter().position(|p| p == &utils_h).unwrap();
    let main_pos = build_order.iter().position(|p| p == &main_c).unwrap();

    assert!(types_pos < utils_pos, "types.h before utils.h");
    assert!(utils_pos < main_pos, "utils.h before main.c");
}
