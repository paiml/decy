    use super::*;

    use tempfile::TempDir;

    #[test]
    fn test_transpile_simple_function() {
        let c_code = "int add(int a, int b) { return a + b; }";
        let result = transpile(c_code);
        assert!(result.is_ok(), "Transpilation should succeed");

        let rust_code = result.unwrap();
        assert!(rust_code.contains("fn add"), "Should contain function name");
        assert!(rust_code.contains("i32"), "Should contain Rust int type");
    }

    #[test]
    fn test_transpile_with_parameters() {
        let c_code = "int multiply(int x, int y) { return x * y; }";
        let result = transpile(c_code);
        assert!(result.is_ok());

        let rust_code = result.unwrap();
        assert!(rust_code.contains("fn multiply"));
        assert!(rust_code.contains("x"));
        assert!(rust_code.contains("y"));
    }

    #[test]
    fn test_transpile_void_function() {
        let c_code = "void do_nothing() { }";
        let result = transpile(c_code);
        assert!(result.is_ok());

        let rust_code = result.unwrap();
        assert!(rust_code.contains("fn do_nothing"));
    }

    #[test]
    fn test_transpile_with_box_transform_simple() {
        // Simple test without actual malloc (just function structure)
        let c_code = "int get_value() { return 42; }";
        let result = transpile_with_box_transform(c_code);
        assert!(result.is_ok());

        let rust_code = result.unwrap();
        assert!(rust_code.contains("fn get_value"));
    }

    #[test]
    fn test_transpile_empty_input() {
        let c_code = "";
        let result = transpile(c_code);
        // Empty input should parse successfully but produce no functions
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_integration_pipeline() {
        // Test that the full pipeline runs without errors
        let c_code = r#"
            int calculate(int a, int b) {
                int result;
                result = a + b;
                return result;
            }
        "#;
        let result = transpile(c_code);
        assert!(result.is_ok(), "Full pipeline should execute");

        let rust_code = result.unwrap();
        assert!(rust_code.contains("fn calculate"));
        assert!(rust_code.contains("let mut result"));
    }

    #[test]
    fn test_transpile_with_lifetime_annotations() {
        // Test that functions with references get lifetime annotations
        // Note: This test depends on the C parser's ability to handle references
        // For now, we test that the pipeline runs successfully
        let c_code = "int add(int a, int b) { return a + b; }";
        let result = transpile(c_code);
        assert!(result.is_ok(), "Transpilation with lifetime analysis should succeed");

        let rust_code = result.unwrap();
        // Basic transpilation should work
        assert!(rust_code.contains("fn add"));

        // When references are present, lifetime annotations would appear
        // Future: Add a test with actual C pointer parameters to verify '<'a> syntax
    }

    // =========================================================================
    // TranspiledFile tests
    // =========================================================================

    #[test]
    fn test_transpiled_file_new() {
        let file = TranspiledFile::new(
            PathBuf::from("/path/to/source.c"),
            "fn main() {}".to_string(),
            vec![PathBuf::from("/path/to/header.h")],
            vec!["main".to_string(), "helper".to_string()],
            "extern \"C\" {}".to_string(),
        );

        assert_eq!(file.source_path, PathBuf::from("/path/to/source.c"));
        assert_eq!(file.rust_code, "fn main() {}");
        assert_eq!(file.dependencies.len(), 1);
        assert_eq!(file.functions_exported.len(), 2);
        assert_eq!(file.ffi_declarations, "extern \"C\" {}");
    }

    #[test]
    fn test_transpiled_file_empty_fields() {
        let file = TranspiledFile::new(
            PathBuf::from("test.c"),
            String::new(),
            Vec::new(),
            Vec::new(),
            String::new(),
        );

        assert!(file.rust_code.is_empty());
        assert!(file.dependencies.is_empty());
        assert!(file.functions_exported.is_empty());
        assert!(file.ffi_declarations.is_empty());
    }

    // =========================================================================
    // ProjectContext tests
    // =========================================================================

    #[test]
    fn test_project_context_new() {
        let ctx = ProjectContext::new();
        assert!(!ctx.has_type("SomeType"));
        assert!(!ctx.has_function("some_func"));
    }

    #[test]
    fn test_project_context_default() {
        let ctx = ProjectContext::default();
        assert!(!ctx.has_type("SomeType"));
        assert!(!ctx.has_function("some_func"));
    }

    #[test]
    fn test_project_context_add_transpiled_file_with_struct() {
        let mut ctx = ProjectContext::new();
        let file = TranspiledFile::new(
            PathBuf::from("test.c"),
            "pub struct Point { x: i32 }".to_string(),
            Vec::new(),
            vec!["create_point".to_string()],
            String::new(),
        );

        ctx.add_transpiled_file(&file);

        assert!(ctx.has_type("Point"));
        assert!(ctx.has_function("create_point"));
        assert_eq!(ctx.get_function_source("create_point"), Some("test.c"));
    }

    #[test]
    fn test_project_context_add_transpiled_file_enums_not_tracked() {
        // Note: add_transpiled_file only tracks structs, not enums (current implementation)
        let mut ctx = ProjectContext::new();
        let file = TranspiledFile::new(
            PathBuf::from("enums.c"),
            "pub enum Color {\n    Red,\n}".to_string(),
            Vec::new(),
            Vec::new(),
            String::new(),
        );

        ctx.add_transpiled_file(&file);
        // Enums are NOT tracked by current implementation
        assert!(!ctx.has_type("Color"));
    }

    #[test]
    fn test_project_context_has_type_not_found() {
        let ctx = ProjectContext::new();
        assert!(!ctx.has_type("NonExistentType"));
    }

    #[test]
    fn test_project_context_has_function_not_found() {
        let ctx = ProjectContext::new();
        assert!(!ctx.has_function("nonexistent_func"));
    }

    #[test]
    fn test_project_context_get_function_source_not_found() {
        let ctx = ProjectContext::new();
        assert!(ctx.get_function_source("nonexistent").is_none());
    }

    #[test]
    fn test_project_context_extract_type_name_struct() {
        let ctx = ProjectContext::new();
        assert_eq!(ctx.extract_type_name("pub struct Point {"), Some("Point".to_string()));
    }

    #[test]
    fn test_project_context_extract_type_name_enum() {
        let ctx = ProjectContext::new();
        assert_eq!(ctx.extract_type_name("pub enum Color {"), Some("Color".to_string()));
    }

    #[test]
    fn test_project_context_extract_type_name_generic() {
        let ctx = ProjectContext::new();
        // Note: The current implementation preserves generic parameters
        assert_eq!(
            ctx.extract_type_name("pub struct Container<T> {"),
            Some("Container<T>".to_string())
        );
    }

    #[test]
    fn test_project_context_extract_type_name_no_match() {
        let ctx = ProjectContext::new();
        assert_eq!(ctx.extract_type_name("fn main() {"), None);
    }

    #[test]
    fn test_project_context_multiple_files() {
        let mut ctx = ProjectContext::new();

        let file1 = TranspiledFile::new(
            PathBuf::from("types.c"),
            "pub struct TypeA { }".to_string(),
            Vec::new(),
            vec!["func_a".to_string()],
            String::new(),
        );

        let file2 = TranspiledFile::new(
            PathBuf::from("utils.c"),
            "pub struct TypeB { }".to_string(),
            Vec::new(),
            vec!["func_b".to_string()],
            String::new(),
        );

        ctx.add_transpiled_file(&file1);
        ctx.add_transpiled_file(&file2);

        assert!(ctx.has_type("TypeA"));
        assert!(ctx.has_type("TypeB"));
        assert!(ctx.has_function("func_a"));
        assert!(ctx.has_function("func_b"));
        assert_eq!(ctx.get_function_source("func_a"), Some("types.c"));
        assert_eq!(ctx.get_function_source("func_b"), Some("utils.c"));
    }

    // =========================================================================
    // DependencyGraph tests
    // =========================================================================

    #[test]
    fn test_dependency_graph_new() {
        let graph = DependencyGraph::new();
        assert!(graph.is_empty());
        assert_eq!(graph.file_count(), 0);
    }

    #[test]
    fn test_dependency_graph_default() {
        let graph = DependencyGraph::default();
        assert!(graph.is_empty());
    }

    #[test]
    fn test_dependency_graph_add_file() {
        let mut graph = DependencyGraph::new();
        let path = Path::new("test.c");

        graph.add_file(path);

        assert!(!graph.is_empty());
        assert_eq!(graph.file_count(), 1);
        assert!(graph.contains_file(path));
    }

    #[test]
    fn test_dependency_graph_add_file_duplicate() {
        let mut graph = DependencyGraph::new();
        let path = Path::new("test.c");

        graph.add_file(path);
        graph.add_file(path); // Should be a no-op

        assert_eq!(graph.file_count(), 1);
    }

    #[test]
    fn test_dependency_graph_contains_file_not_found() {
        let graph = DependencyGraph::new();
        assert!(!graph.contains_file(Path::new("nonexistent.c")));
    }

    #[test]
    fn test_dependency_graph_add_dependency() {
        let mut graph = DependencyGraph::new();
        let main_path = Path::new("main.c");
        let header_path = Path::new("header.h");

        graph.add_file(main_path);
        graph.add_file(header_path);
        graph.add_dependency(main_path, header_path);

        assert!(graph.has_dependency(main_path, header_path));
        assert!(!graph.has_dependency(header_path, main_path));
    }

    #[test]
    fn test_dependency_graph_has_dependency_missing_files() {
        let graph = DependencyGraph::new();
        assert!(!graph.has_dependency(Path::new("a.c"), Path::new("b.c")));
    }

    #[test]
    fn test_dependency_graph_topological_sort_simple() {
        let mut graph = DependencyGraph::new();
        let main_path = PathBuf::from("main.c");
        let utils_path = PathBuf::from("utils.c");
        let header_path = PathBuf::from("header.h");

        graph.add_file(&main_path);
        graph.add_file(&utils_path);
        graph.add_file(&header_path);

        // main depends on utils, utils depends on header
        graph.add_dependency(&main_path, &utils_path);
        graph.add_dependency(&utils_path, &header_path);

        let order = graph.topological_sort().unwrap();

        // header should come before utils, utils before main
        let header_idx = order.iter().position(|p| p == &header_path).unwrap();
        let utils_idx = order.iter().position(|p| p == &utils_path).unwrap();
        let main_idx = order.iter().position(|p| p == &main_path).unwrap();

        assert!(header_idx < utils_idx);
        assert!(utils_idx < main_idx);
    }

    #[test]
    fn test_dependency_graph_topological_sort_circular() {
        let mut graph = DependencyGraph::new();
        let a_path = PathBuf::from("a.c");
        let b_path = PathBuf::from("b.c");

        graph.add_file(&a_path);
        graph.add_file(&b_path);

        // Create a cycle: a -> b -> a
        graph.add_dependency(&a_path, &b_path);
        graph.add_dependency(&b_path, &a_path);

        let result = graph.topological_sort();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circular dependency"));
    }

    #[test]
    fn test_dependency_graph_topological_sort_empty() {
        let graph = DependencyGraph::new();
        let order = graph.topological_sort().unwrap();
        assert!(order.is_empty());
    }

    #[test]
    fn test_dependency_graph_parse_include_directives() {
        let code = r#"
            #include <stdio.h>
            #include "myheader.h"
            #include <stdlib.h>
            int main() { return 0; }
        "#;

        let includes = DependencyGraph::parse_include_directives(code);

        assert_eq!(includes.len(), 3);
        assert!(includes.contains(&"stdio.h".to_string()));
        assert!(includes.contains(&"myheader.h".to_string()));
        assert!(includes.contains(&"stdlib.h".to_string()));
    }

    #[test]
    fn test_dependency_graph_parse_include_directives_empty() {
        let code = "int main() { return 0; }";
        let includes = DependencyGraph::parse_include_directives(code);
        assert!(includes.is_empty());
    }

    #[test]
    fn test_dependency_graph_parse_include_directives_malformed() {
        let code = r#"
            #include
            #include "
            #include <
            int main() { return 0; }
        "#;

        let includes = DependencyGraph::parse_include_directives(code);
        assert!(includes.is_empty());
    }

    #[test]
    fn test_dependency_graph_has_header_guard() {
        let temp_dir = TempDir::new().unwrap();
        let header_path = temp_dir.path().join("guarded.h");

        let header_content = r#"
            #ifndef GUARDED_H
            #define GUARDED_H
            int foo();
            #endif
        "#;

        std::fs::write(&header_path, header_content).unwrap();

        assert!(DependencyGraph::has_header_guard(&header_path).unwrap());
    }

    #[test]
    fn test_dependency_graph_has_header_guard_if_not_defined() {
        let temp_dir = TempDir::new().unwrap();
        let header_path = temp_dir.path().join("guarded2.h");

        let header_content = r#"
            #if !defined(GUARDED2_H)
            #define GUARDED2_H
            int bar();
            #endif
        "#;

        std::fs::write(&header_path, header_content).unwrap();

        assert!(DependencyGraph::has_header_guard(&header_path).unwrap());
    }

    #[test]
    fn test_dependency_graph_has_header_guard_missing_guard() {
        let temp_dir = TempDir::new().unwrap();
        let header_path = temp_dir.path().join("unguarded.h");

        let header_content = r#"
            int baz();
        "#;

        std::fs::write(&header_path, header_content).unwrap();

        assert!(!DependencyGraph::has_header_guard(&header_path).unwrap());
    }

    #[test]
    fn test_dependency_graph_has_header_guard_file_not_found() {
        let result = DependencyGraph::has_header_guard(Path::new("/nonexistent/file.h"));
        assert!(result.is_err());
    }

    #[test]
    fn test_dependency_graph_from_files() {
        let temp_dir = TempDir::new().unwrap();

        let header_path = temp_dir.path().join("header.h");
        std::fs::write(&header_path, "int helper();").unwrap();

        let main_path = temp_dir.path().join("main.c");
        std::fs::write(
            &main_path,
            r#"#include "header.h"
            int main() { return helper(); }"#,
        )
        .unwrap();

        let graph = DependencyGraph::from_files(&[main_path.clone(), header_path.clone()]).unwrap();

        assert_eq!(graph.file_count(), 2);
        assert!(graph.has_dependency(&main_path, &header_path));
    }

    #[test]
    fn test_dependency_graph_from_files_nonexistent() {
        let result = DependencyGraph::from_files(&[PathBuf::from("/nonexistent/file.c")]);
        assert!(result.is_err());
    }

    // =========================================================================
    // TranspilationCache tests
    // =========================================================================

    #[test]
    fn test_transpilation_cache_new() {
        let cache = TranspilationCache::new();
        let stats = cache.statistics();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.total_files, 0);
    }

    #[test]
    fn test_transpilation_cache_default() {
        let cache = TranspilationCache::default();
        assert_eq!(cache.statistics().total_files, 0);
    }

    #[test]
    fn test_transpilation_cache_with_directory() {
        let temp_dir = TempDir::new().unwrap();
        let cache = TranspilationCache::with_directory(temp_dir.path());
        assert_eq!(cache.statistics().total_files, 0);
    }

    #[test]
    fn test_transpilation_cache_compute_hash() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.c");
        std::fs::write(&file_path, "int main() { return 0; }").unwrap();

        let cache = TranspilationCache::new();
        let hash = cache.compute_hash(&file_path).unwrap();

        // SHA-256 hash is 64 hex characters
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_transpilation_cache_compute_hash_file_not_found() {
        let cache = TranspilationCache::new();
        let result = cache.compute_hash(Path::new("/nonexistent/file.c"));
        assert!(result.is_err());
    }

    #[test]
    fn test_transpilation_cache_insert_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.c");
        std::fs::write(&file_path, "int main() { return 0; }").unwrap();

        let mut cache = TranspilationCache::new();
        let transpiled = TranspiledFile::new(
            file_path.clone(),
            "fn main() -> i32 { 0 }".to_string(),
            Vec::new(),
            vec!["main".to_string()],
            String::new(),
        );

        cache.insert(&file_path, &transpiled);

        let cached = cache.get(&file_path);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().rust_code, "fn main() -> i32 { 0 }");
    }

    #[test]
    fn test_transpilation_cache_get_miss_on_file_change() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.c");
        std::fs::write(&file_path, "int main() { return 0; }").unwrap();

        let mut cache = TranspilationCache::new();
        let transpiled = TranspiledFile::new(
            file_path.clone(),
            "fn main() -> i32 { 0 }".to_string(),
            Vec::new(),
            Vec::new(),
            String::new(),
        );

        cache.insert(&file_path, &transpiled);

        // Modify the file
        std::fs::write(&file_path, "int main() { return 42; }").unwrap();

        let cached = cache.get(&file_path);
        assert!(cached.is_none()); // Cache miss due to file change
    }

    #[test]
    fn test_transpilation_cache_get_miss_on_dependency_change() {
        let temp_dir = TempDir::new().unwrap();
        let main_path = temp_dir.path().join("main.c");
        let dep_path = temp_dir.path().join("dep.h");

        std::fs::write(&main_path, "#include \"dep.h\"\nint main() { return foo(); }").unwrap();
        std::fs::write(&dep_path, "int foo();").unwrap();

        let mut cache = TranspilationCache::new();
        let transpiled = TranspiledFile::new(
            main_path.clone(),
            "fn main() -> i32 { foo() }".to_string(),
            vec![dep_path.clone()],
            vec!["main".to_string()],
            String::new(),
        );

        cache.insert(&main_path, &transpiled);

        // Modify the dependency
        std::fs::write(&dep_path, "int foo() { return 42; }").unwrap();

        let cached = cache.get(&main_path);
        assert!(cached.is_none()); // Cache miss due to dependency change
    }

    #[test]
    fn test_transpilation_cache_clear() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.c");
        std::fs::write(&file_path, "int main() { return 0; }").unwrap();

        let mut cache = TranspilationCache::new();
        let transpiled = TranspiledFile::new(
            file_path.clone(),
            "fn main() -> i32 { 0 }".to_string(),
            Vec::new(),
            Vec::new(),
            String::new(),
        );

        cache.insert(&file_path, &transpiled);
        assert_eq!(cache.statistics().total_files, 1);

        cache.clear();
        assert_eq!(cache.statistics().total_files, 0);
        assert_eq!(cache.statistics().hits, 0);
        assert_eq!(cache.statistics().misses, 0);
    }

    #[test]
    fn test_transpilation_cache_statistics() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.c");
        std::fs::write(&file_path, "int main() { return 0; }").unwrap();

        let mut cache = TranspilationCache::new();
        let transpiled = TranspiledFile::new(
            file_path.clone(),
            "fn main() -> i32 { 0 }".to_string(),
            Vec::new(),
            Vec::new(),
            String::new(),
        );

        cache.insert(&file_path, &transpiled);

        // Cache hit
        let _ = cache.get(&file_path);
        let stats = cache.statistics();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.total_files, 1);
    }

    #[test]
    fn test_transpilation_cache_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let file_path = temp_dir.path().join("test.c");
        std::fs::write(&file_path, "int main() { return 0; }").unwrap();

        // Create and populate cache
        let mut cache = TranspilationCache::with_directory(&cache_dir);
        let transpiled = TranspiledFile::new(
            file_path.clone(),
            "fn main() -> i32 { 0 }".to_string(),
            Vec::new(),
            vec!["main".to_string()],
            String::new(),
        );
        cache.insert(&file_path, &transpiled);

        // Save
        cache.save().unwrap();
        assert!(cache_dir.join("cache.json").exists());

        // Load into new cache
        let loaded_cache = TranspilationCache::load(&cache_dir).unwrap();
        assert_eq!(loaded_cache.statistics().total_files, 1);
    }

    #[test]
    fn test_transpilation_cache_save_no_directory() {
        let cache = TranspilationCache::new();
        let result = cache.save();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not set"));
    }

    #[test]
    fn test_transpilation_cache_load_no_file() {
        let temp_dir = TempDir::new().unwrap();
        let cache = TranspilationCache::load(temp_dir.path()).unwrap();
        assert_eq!(cache.statistics().total_files, 0);
    }

    // =========================================================================
    // Helper function tests
    // =========================================================================

    #[test]
    fn test_extract_function_names() {
        let rust_code = r#"
            fn add(a: i32, b: i32) -> i32 { a + b }
            pub fn multiply(x: i32, y: i32) -> i32 { x * y }
            fn foo<'a>(s: &'a str) -> &'a str { s }
            struct Point { x: i32, y: i32 }
        "#;

        let names = extract_function_names(rust_code);

        assert_eq!(names.len(), 3);
        assert!(names.contains(&"add".to_string()));
        assert!(names.contains(&"multiply".to_string()));
        assert!(names.contains(&"foo".to_string()));
    }

    #[test]
    fn test_extract_function_names_empty() {
        let rust_code = "struct Point { x: i32 }";
        let names = extract_function_names(rust_code);
        assert!(names.is_empty());
    }

    #[test]
    fn test_generate_ffi_declarations() {
        let functions = vec!["add".to_string(), "multiply".to_string()];
        let ffi = generate_ffi_declarations(&functions);

        assert!(ffi.contains("extern \"C\""));
        assert!(ffi.contains("// add"));
        assert!(ffi.contains("// multiply"));
    }

    #[test]
    fn test_generate_ffi_declarations_empty() {
        let functions: Vec<String> = Vec::new();
        let ffi = generate_ffi_declarations(&functions);
        assert!(ffi.is_empty());
    }

    #[test]
    fn test_extract_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let header_path = temp_dir.path().join("myheader.h");
        std::fs::write(&header_path, "int helper();").unwrap();

        let source_path = temp_dir.path().join("main.c");
        std::fs::write(
            &source_path,
            r#"#include "myheader.h"
            #include <stdio.h>
            int main() { return 0; }"#,
        )
        .unwrap();

        let c_code = std::fs::read_to_string(&source_path).unwrap();
        let deps = extract_dependencies(&source_path, &c_code).unwrap();

        // Only local includes that exist should be in deps
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], header_path);
    }

    #[test]
    fn test_extract_dependencies_no_parent() {
        // This shouldn't happen in practice but we test the error case
        // The function expects a file with a parent directory
        let c_code = "#include \"header.h\"";
        let result = extract_dependencies(Path::new(""), c_code);
        assert!(result.is_err());
    }

    // =========================================================================
    // Pointer arithmetic and NULL comparison tests
    // =========================================================================

    #[test]
    fn test_uses_pointer_arithmetic_true() {
        use decy_hir::{BinaryOperator, HirParameter, HirType};

        let params =
            vec![HirParameter::new("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)))];

        // Add statement: ptr = ptr + 1
        let body = vec![HirStatement::Assignment {
            target: "ptr".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("ptr".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }];

        let func = HirFunction::new_with_body("test".to_string(), HirType::Void, params, body);

        assert!(uses_pointer_arithmetic(&func, "ptr"));
        assert!(!uses_pointer_arithmetic(&func, "other"));
    }

    #[test]
    fn test_uses_pointer_arithmetic_subtract() {
        use decy_hir::{BinaryOperator, HirParameter, HirType};

        let params =
            vec![HirParameter::new("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)))];

        // Add statement: ptr = ptr - 1
        let body = vec![HirStatement::Assignment {
            target: "ptr".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Subtract,
                left: Box::new(HirExpression::Variable("ptr".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }];

        let func = HirFunction::new_with_body("test".to_string(), HirType::Void, params, body);

        assert!(uses_pointer_arithmetic(&func, "ptr"));
    }

    #[test]
    fn test_uses_pointer_arithmetic_in_if() {
        use decy_hir::{BinaryOperator, HirParameter, HirType};

        let params =
            vec![HirParameter::new("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)))];

        // Add statement inside if block
        let body = vec![HirStatement::If {
            condition: HirExpression::IntLiteral(1), // true condition
            then_block: vec![HirStatement::Assignment {
                target: "ptr".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("ptr".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            }],
            else_block: None,
        }];

        let func = HirFunction::new_with_body("test".to_string(), HirType::Void, params, body);

        assert!(uses_pointer_arithmetic(&func, "ptr"));
    }

    #[test]
    fn test_uses_pointer_arithmetic_in_else() {
        use decy_hir::{BinaryOperator, HirParameter, HirType};

        let params =
            vec![HirParameter::new("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)))];

        let body = vec![HirStatement::If {
            condition: HirExpression::IntLiteral(1), // true condition
            then_block: vec![],
            else_block: Some(vec![HirStatement::Assignment {
                target: "ptr".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("ptr".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            }]),
        }];

        let func = HirFunction::new_with_body("test".to_string(), HirType::Void, params, body);

        assert!(uses_pointer_arithmetic(&func, "ptr"));
    }

    #[test]
    fn test_uses_pointer_arithmetic_in_while() {
        use decy_hir::{BinaryOperator, HirParameter, HirType};

        let params =
            vec![HirParameter::new("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)))];

        let body = vec![HirStatement::While {
            condition: HirExpression::IntLiteral(1), // true condition
            body: vec![HirStatement::Assignment {
                target: "ptr".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("ptr".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            }],
        }];

        let func = HirFunction::new_with_body("test".to_string(), HirType::Void, params, body);

        assert!(uses_pointer_arithmetic(&func, "ptr"));
    }

    #[test]
    fn test_uses_pointer_arithmetic_in_for() {
        use decy_hir::{BinaryOperator, HirParameter, HirType};

        let params =
            vec![HirParameter::new("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)))];

        let body = vec![HirStatement::For {
            init: vec![],
            condition: Some(HirExpression::IntLiteral(1)), // true condition
            increment: vec![],
            body: vec![HirStatement::Assignment {
                target: "ptr".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("ptr".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            }],
        }];

        let func = HirFunction::new_with_body("test".to_string(), HirType::Void, params, body);

        assert!(uses_pointer_arithmetic(&func, "ptr"));
    }

    #[test]
    fn test_uses_pointer_arithmetic_false() {
        use decy_hir::{HirParameter, HirType};

        let params =
            vec![HirParameter::new("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int)))];

        // Simple assignment without pointer arithmetic
        let body = vec![HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::IntLiteral(42),
        }];

        let func = HirFunction::new_with_body("test".to_string(), HirType::Void, params, body);

        assert!(!uses_pointer_arithmetic(&func, "ptr"));
    }

    #[test]
    fn test_pointer_compared_to_null_equal() {
        use decy_hir::{BinaryOperator, HirType};

        let body = vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::Equal,
                left: Box::new(HirExpression::Variable("ptr".to_string())),
                right: Box::new(HirExpression::NullLiteral),
            },
            then_block: vec![],
            else_block: None,
        }];

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![decy_hir::HirParameter::new(
                "ptr".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            )],
            body,
        );

        assert!(pointer_compared_to_null(&func, "ptr"));
        assert!(!pointer_compared_to_null(&func, "other"));
    }

    #[test]
    fn test_pointer_compared_to_null_not_equal() {
        use decy_hir::{BinaryOperator, HirType};

        let body = vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::NotEqual,
                left: Box::new(HirExpression::Variable("ptr".to_string())),
                right: Box::new(HirExpression::NullLiteral),
            },
            then_block: vec![],
            else_block: None,
        }];

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![decy_hir::HirParameter::new(
                "ptr".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            )],
            body,
        );

        assert!(pointer_compared_to_null(&func, "ptr"));
    }

    #[test]
    fn test_pointer_compared_to_null_zero_literal() {
        use decy_hir::{BinaryOperator, HirType};

        // NULL can be represented as IntLiteral(0)
        let body = vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::Equal,
                left: Box::new(HirExpression::Variable("ptr".to_string())),
                right: Box::new(HirExpression::IntLiteral(0)),
            },
            then_block: vec![],
            else_block: None,
        }];

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![decy_hir::HirParameter::new(
                "ptr".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            )],
            body,
        );

        assert!(pointer_compared_to_null(&func, "ptr"));
    }

    #[test]
    fn test_pointer_compared_to_null_in_while() {
        use decy_hir::{BinaryOperator, HirType};

        let body = vec![HirStatement::While {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::NotEqual,
                left: Box::new(HirExpression::Variable("ptr".to_string())),
                right: Box::new(HirExpression::NullLiteral),
            },
            body: vec![],
        }];

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![decy_hir::HirParameter::new(
                "ptr".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            )],
            body,
        );

        assert!(pointer_compared_to_null(&func, "ptr"));
    }

    #[test]
    fn test_pointer_compared_to_null_in_for() {
        use decy_hir::{BinaryOperator, HirType};

        let body = vec![HirStatement::For {
            init: vec![],
            condition: Some(HirExpression::BinaryOp {
                op: BinaryOperator::NotEqual,
                left: Box::new(HirExpression::Variable("ptr".to_string())),
                right: Box::new(HirExpression::NullLiteral),
            }),
            increment: vec![],
            body: vec![],
        }];

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![decy_hir::HirParameter::new(
                "ptr".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            )],
            body,
        );

        assert!(pointer_compared_to_null(&func, "ptr"));
    }

    #[test]
    fn test_pointer_compared_to_null_in_switch() {
        use decy_hir::{BinaryOperator, HirType, SwitchCase};

        let body = vec![HirStatement::Switch {
            condition: HirExpression::Variable("x".to_string()),
            cases: vec![SwitchCase {
                value: Some(HirExpression::IntLiteral(1)),
                body: vec![HirStatement::If {
                    condition: HirExpression::BinaryOp {
                        op: BinaryOperator::Equal,
                        left: Box::new(HirExpression::Variable("ptr".to_string())),
                        right: Box::new(HirExpression::NullLiteral),
                    },
                    then_block: vec![],
                    else_block: None,
                }],
            }],
            default_case: None,
        }];

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![decy_hir::HirParameter::new(
                "ptr".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            )],
            body,
        );

        assert!(pointer_compared_to_null(&func, "ptr"));
    }

    #[test]
    fn test_pointer_compared_to_null_reversed() {
        use decy_hir::{BinaryOperator, HirType};

        // NULL on left side: NULL == ptr
        let body = vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::Equal,
                left: Box::new(HirExpression::NullLiteral),
                right: Box::new(HirExpression::Variable("ptr".to_string())),
            },
            then_block: vec![],
            else_block: None,
        }];

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![decy_hir::HirParameter::new(
                "ptr".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            )],
            body,
        );

        assert!(pointer_compared_to_null(&func, "ptr"));
    }

    #[test]
    fn test_pointer_compared_to_null_nested_binary_op() {
        use decy_hir::{BinaryOperator, HirType};

        // (ptr == NULL) && (other == NULL)
        let body = vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::LogicalAnd,
                left: Box::new(HirExpression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(HirExpression::Variable("ptr".to_string())),
                    right: Box::new(HirExpression::NullLiteral),
                }),
                right: Box::new(HirExpression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(HirExpression::Variable("other".to_string())),
                    right: Box::new(HirExpression::NullLiteral),
                }),
            },
            then_block: vec![],
            else_block: None,
        }];

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![
                decy_hir::HirParameter::new(
                    "ptr".to_string(),
                    HirType::Pointer(Box::new(HirType::Int)),
                ),
                decy_hir::HirParameter::new(
                    "other".to_string(),
                    HirType::Pointer(Box::new(HirType::Int)),
                ),
            ],
            body,
        );

        assert!(pointer_compared_to_null(&func, "ptr"));
        assert!(pointer_compared_to_null(&func, "other"));
    }

    #[test]
    fn test_pointer_not_compared_to_null() {
        use decy_hir::HirType;

        let body = vec![HirStatement::Expression(HirExpression::Variable("ptr".to_string()))];

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![decy_hir::HirParameter::new(
                "ptr".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            )],
            body,
        );

        assert!(!pointer_compared_to_null(&func, "ptr"));
    }

    // =========================================================================
    // transpile_with_verification tests
    // =========================================================================

    #[test]
    fn test_transpile_with_verification_success() {
        let c_code = "int add(int a, int b) { return a + b; }";
        let result = transpile_with_verification(c_code).unwrap();

        assert!(!result.rust_code.is_empty());
        assert!(result.rust_code.contains("fn add"));
    }

    #[test]
    fn test_transpile_with_verification_failure() {
        // Invalid C code
        let c_code = "int add( { }"; // Malformed
        let result = transpile_with_verification(c_code).unwrap();

        // Should return a result with empty code and errors
        assert!(result.rust_code.is_empty() || !result.errors.is_empty());
    }

    // =========================================================================
    // transpile_file tests
    // =========================================================================

    #[test]
    fn test_transpile_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.c");
        std::fs::write(&file_path, "int add(int a, int b) { return a + b; }").unwrap();

        let ctx = ProjectContext::new();
        let result = transpile_file(&file_path, &ctx).unwrap();

        assert_eq!(result.source_path, file_path);
        assert!(result.rust_code.contains("fn add"));
        assert!(result.functions_exported.contains(&"add".to_string()));
    }

    #[test]
    fn test_transpile_file_not_found() {
        let ctx = ProjectContext::new();
        let result = transpile_file(Path::new("/nonexistent/file.c"), &ctx);
        assert!(result.is_err());
    }

    // =========================================================================
    // transpile_from_file_path tests
    // =========================================================================

    #[test]
    fn test_transpile_from_file_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.c");
        std::fs::write(&file_path, "int multiply(int x, int y) { return x * y; }").unwrap();

        let result = transpile_from_file_path(&file_path).unwrap();
        assert!(result.contains("fn multiply"));
    }

    #[test]
    fn test_transpile_from_file_path_not_found() {
        let result = transpile_from_file_path(Path::new("/nonexistent/file.c"));
        assert!(result.is_err());
    }

    // =========================================================================
    // transpile_with_includes tests
    // =========================================================================

    #[test]
    fn test_transpile_with_includes_system_header() {
        // System headers get commented out and replaced with stdlib prototypes
        let c_code = r#"
            #include <stdio.h>
            int main() { return 0; }
        "#;

        let result = transpile_with_includes(c_code, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_with_includes_local_header() {
        let temp_dir = TempDir::new().unwrap();
        let header_path = temp_dir.path().join("myheader.h");
        std::fs::write(&header_path, "int helper(int x);").unwrap();

        let c_code = r#"
            #include "myheader.h"
            int main() { return helper(42); }
        "#;

        let result = transpile_with_includes(c_code, Some(temp_dir.path()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_with_includes_missing_local_header() {
        let c_code = r#"
            #include "nonexistent.h"
            int main() { return 0; }
        "#;

        let result = transpile_with_includes(c_code, None);
        assert!(result.is_err());
    }

    // =========================================================================
    // struct and enum transpilation tests
    // =========================================================================

    #[test]
    fn test_transpile_struct() {
        let c_code = r#"
            struct Point {
                int x;
                int y;
            };
            int get_x(struct Point* p) { return p->x; }
        "#;

        let result = transpile(c_code).unwrap();
        assert!(result.contains("struct Point"));
        assert!(result.contains("x: i32"));
    }

    #[test]
    fn test_transpile_enum() {
        let c_code = r#"
            enum Color { RED, GREEN, BLUE };
            int main() { return RED; }
        "#;

        let result = transpile(c_code).unwrap();
        // Enums are transpiled as const i32 values
        assert!(result.contains("RED") || result.contains("Color"));
    }

    #[test]
    fn test_transpile_global_variable() {
        let c_code = r#"
            int global_counter = 0;
            void increment() { global_counter = global_counter + 1; }
        "#;

        let result = transpile(c_code).unwrap();
        assert!(result.contains("static mut global_counter"));
    }

    #[test]
    fn test_transpile_typedef() {
        let c_code = r#"
            typedef int MyInt;
            MyInt add(MyInt a, MyInt b) { return a + b; }
        "#;

        let result = transpile(c_code).unwrap();
        // Should contain typedef or the underlying type
        assert!(result.contains("fn add"));
    }

    // =========================================================================
    // expression_compares_to_null tests (internal function coverage)
    // =========================================================================

    #[test]
    fn test_expression_compares_to_null_unary_op() {
        use decy_hir::{BinaryOperator, UnaryOperator};

        // !((ptr == NULL))
        let expr = HirExpression::UnaryOp {
            op: UnaryOperator::LogicalNot,
            operand: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Equal,
                left: Box::new(HirExpression::Variable("ptr".to_string())),
                right: Box::new(HirExpression::NullLiteral),
            }),
        };

        assert!(expression_compares_to_null(&expr, "ptr"));
    }

    #[test]
    fn test_expression_compares_to_null_other_expression() {
        // Variable expression (not a comparison)
        let expr = HirExpression::Variable("ptr".to_string());
        assert!(!expression_compares_to_null(&expr, "ptr"));

        // Int literal
        let expr = HirExpression::IntLiteral(42);
        assert!(!expression_compares_to_null(&expr, "ptr"));

        // Function call
        let expr = HirExpression::FunctionCall { function: "foo".to_string(), arguments: vec![] };
        assert!(!expression_compares_to_null(&expr, "ptr"));
    }

    // =========================================================================
    // statement_compares_to_null additional branch tests
    // =========================================================================

    #[test]
    fn test_statement_compares_to_null_nested_in_then_block() {
        use decy_hir::BinaryOperator;

        let stmt = HirStatement::If {
            condition: HirExpression::IntLiteral(1), // true condition
            then_block: vec![HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(HirExpression::Variable("ptr".to_string())),
                    right: Box::new(HirExpression::NullLiteral),
                },
                then_block: vec![],
                else_block: None,
            }],
            else_block: None,
        };

        assert!(statement_compares_to_null(&stmt, "ptr"));
    }

    #[test]
    fn test_statement_compares_to_null_nested_in_else_block() {
        use decy_hir::BinaryOperator;

        let stmt = HirStatement::If {
            condition: HirExpression::IntLiteral(1), // true condition
            then_block: vec![],
            else_block: Some(vec![HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(HirExpression::Variable("ptr".to_string())),
                    right: Box::new(HirExpression::NullLiteral),
                },
                then_block: vec![],
                else_block: None,
            }]),
        };

        assert!(statement_compares_to_null(&stmt, "ptr"));
    }

    #[test]
    fn test_statement_compares_to_null_in_while_body() {
        use decy_hir::BinaryOperator;

        let stmt = HirStatement::While {
            condition: HirExpression::IntLiteral(1), // true condition
            body: vec![HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(HirExpression::Variable("ptr".to_string())),
                    right: Box::new(HirExpression::NullLiteral),
                },
                then_block: vec![],
                else_block: None,
            }],
        };

        assert!(statement_compares_to_null(&stmt, "ptr"));
    }

    #[test]
    fn test_statement_compares_to_null_in_for_body() {
        use decy_hir::BinaryOperator;

        let stmt = HirStatement::For {
            init: vec![],
            condition: Some(HirExpression::IntLiteral(1)), // true condition
            increment: vec![],
            body: vec![HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(HirExpression::Variable("ptr".to_string())),
                    right: Box::new(HirExpression::NullLiteral),
                },
                then_block: vec![],
                else_block: None,
            }],
        };

        assert!(statement_compares_to_null(&stmt, "ptr"));
    }

    #[test]
    fn test_statement_compares_to_null_return_statement() {
        // Return statement doesn't contain comparisons
        let stmt = HirStatement::Return(Some(HirExpression::IntLiteral(0)));
        assert!(!statement_compares_to_null(&stmt, "ptr"));
    }

    #[test]
    fn test_statement_compares_to_null_expression_statement() {
        // Expression statement
        let stmt = HirStatement::Expression(HirExpression::Variable("x".to_string()));
        assert!(!statement_compares_to_null(&stmt, "ptr"));
    }

    // ========================================================================
    // transpile_with_trace coverage
    // ========================================================================

    #[test]
    fn test_transpile_with_trace_basic() {
        let c_code = "int add(int a, int b) { return a + b; }";
        let result = transpile_with_trace(c_code);
        assert!(result.is_ok(), "transpile_with_trace should succeed");
        let (rust_code, collector) = result.unwrap();
        assert!(rust_code.contains("fn add"));
        let entries = collector.entries();
        assert!(entries.len() >= 2, "Should have parsing + completion entries");
    }

    // ========================================================================
    // transpile_with_verification coverage (additional)
    // ========================================================================

    #[test]
    fn test_transpile_with_verification_empty_code() {
        // Empty code should still produce some result
        let result = transpile_with_verification("");
        assert!(result.is_ok());
    }

    // ========================================================================
    // extract_function_names coverage
    // ========================================================================

    #[test]
    fn test_extract_function_names_basic() {
        let code = "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}";
        let names = extract_function_names(code);
        assert!(names.contains(&"add".to_string()));
    }

    #[test]
    fn test_extract_function_names_pub_fn() {
        let code = "pub fn multiply(x: i32, y: i32) -> i32 {\n    x * y\n}";
        let names = extract_function_names(code);
        assert!(names.contains(&"multiply".to_string()));
    }

    #[test]
    fn test_extract_function_names_generic() {
        let code = "fn process<'a>(data: &'a [i32]) -> &'a i32 {\n    &data[0]\n}";
        let names = extract_function_names(code);
        assert!(names.contains(&"process".to_string()));
    }

    #[test]
    fn test_extract_function_names_multiple() {
        let code = "fn foo(x: i32) {}\npub fn bar(y: f64) {}\nfn baz() {}";
        let names = extract_function_names(code);
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"foo".to_string()));
        assert!(names.contains(&"bar".to_string()));
        assert!(names.contains(&"baz".to_string()));
    }

    #[test]
    fn test_extract_function_names_no_functions() {
        let code = "let x = 42;\nstruct Point { x: i32 }";
        let names = extract_function_names(code);
        assert!(names.is_empty());
    }

    // ========================================================================
    // generate_ffi_declarations coverage
    // ========================================================================

    #[test]
    fn test_generate_ffi_declarations_with_functions() {
        let functions = vec!["add".to_string(), "multiply".to_string()];
        let ffi = generate_ffi_declarations(&functions);
        assert!(ffi.contains("extern \"C\""));
        assert!(ffi.contains("add"));
        assert!(ffi.contains("multiply"));
    }

    // ========================================================================
    // extract_dependencies coverage
    // ========================================================================

    #[test]
    fn test_extract_dependencies_no_includes() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("main.c");
        std::fs::write(&src, "int main() { return 0; }").unwrap();
        let deps = extract_dependencies(&src, "int main() { return 0; }").unwrap();
        assert!(deps.is_empty());
    }

    #[test]
    fn test_extract_dependencies_with_includes() {
        let tmp = TempDir::new().unwrap();
        let header = tmp.path().join("utils.h");
        std::fs::write(&header, "int helper(int x);").unwrap();
        let src = tmp.path().join("main.c");
        let c_code = "#include \"utils.h\"\nint main() { return 0; }";
        std::fs::write(&src, c_code).unwrap();
        let deps = extract_dependencies(&src, c_code).unwrap();
        assert_eq!(deps.len(), 1);
        assert!(deps[0].ends_with("utils.h"));
    }

    #[test]
    fn test_extract_dependencies_missing_header() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("main.c");
        let c_code = "#include \"nonexistent.h\"\nint main() { return 0; }";
        std::fs::write(&src, c_code).unwrap();
        let deps = extract_dependencies(&src, c_code).unwrap();
        assert!(deps.is_empty()); // File doesn't exist, so not included
    }

    // ========================================================================
    // uses_pointer_arithmetic coverage
    // ========================================================================

    #[test]
    fn test_uses_pointer_arithmetic_empty_body() {
        use decy_hir::{HirParameter, HirType};
        let func = HirFunction::new(
            "test".to_string(),
            HirType::Void,
            vec![HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Int)))],
        );
        assert!(!uses_pointer_arithmetic(&func, "p"));
    }

    #[test]
    fn test_uses_pointer_arithmetic_with_increment() {
        use decy_hir::{BinaryOperator, HirParameter, HirType};
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![HirParameter::new("p".to_string(), HirType::Pointer(Box::new(HirType::Int)))],
            vec![HirStatement::Assignment {
                target: "p".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("p".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            }],
        );
        assert!(uses_pointer_arithmetic(&func, "p"));
    }

    // ========================================================================
    // statement_compares_to_null: switch and for(;;)
    // ========================================================================

    #[test]
    fn test_statement_compares_to_null_switch() {
        let stmt = HirStatement::Switch {
            condition: HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Equal,
                left: Box::new(HirExpression::Variable("ptr".to_string())),
                right: Box::new(HirExpression::NullLiteral),
            },
            cases: vec![],
            default_case: None,
        };
        assert!(statement_compares_to_null(&stmt, "ptr"));
    }

    #[test]
    fn test_statement_compares_to_null_for_none_condition() {
        // for(;;) with no condition — should not match null comparison
        let stmt =
            HirStatement::For { init: vec![], condition: None, increment: vec![], body: vec![] };
        assert!(!statement_compares_to_null(&stmt, "ptr"));
    }

    #[test]
    fn test_statement_compares_to_null_for_with_null_in_body() {
        let stmt = HirStatement::For {
            init: vec![],
            condition: Some(HirExpression::IntLiteral(1)),
            increment: vec![],
            body: vec![HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Equal,
                    left: Box::new(HirExpression::Variable("ptr".to_string())),
                    right: Box::new(HirExpression::NullLiteral),
                },
                then_block: vec![HirStatement::Break],
                else_block: None,
            }],
        };
        assert!(statement_compares_to_null(&stmt, "ptr"));
    }
