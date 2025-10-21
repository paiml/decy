//! Core transpilation pipeline for C-to-Rust conversion.
//!
//! This crate orchestrates the entire transpilation process:
//! 1. Parse C code (via decy-parser)
//! 2. Convert to HIR (via decy-hir)
//! 3. Analyze and infer types (via decy-analyzer)
//! 4. Infer ownership and lifetimes (via decy-ownership)
//! 5. Verify safety properties (via decy-verify)
//! 6. Generate Rust code (via decy-codegen)

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

use anyhow::{Context, Result};
use decy_analyzer::patterns::PatternDetector;
use decy_codegen::CodeGenerator;
use decy_hir::HirFunction;
use decy_ownership::{
    borrow_gen::BorrowGenerator, dataflow::DataflowAnalyzer, inference::OwnershipInferencer,
    lifetime::LifetimeAnalyzer, lifetime_gen::LifetimeAnnotator,
};
use decy_parser::parser::CParser;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::Topo;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Result of transpiling a single C file.
///
/// Contains the transpiled Rust code along with metadata about
/// dependencies and exported symbols for cross-file reference tracking.
#[derive(Debug, Clone)]
pub struct TranspiledFile {
    /// Path to the original C source file
    pub source_path: PathBuf,

    /// Generated Rust code
    pub rust_code: String,

    /// List of C files this file depends on (#include dependencies)
    pub dependencies: Vec<PathBuf>,

    /// Functions exported by this file (for FFI and cross-file references)
    pub functions_exported: Vec<String>,

    /// FFI declarations (extern "C") for C↔Rust boundaries
    pub ffi_declarations: String,
}

impl TranspiledFile {
    /// Create a new TranspiledFile with the given data.
    pub fn new(
        source_path: PathBuf,
        rust_code: String,
        dependencies: Vec<PathBuf>,
        functions_exported: Vec<String>,
        ffi_declarations: String,
    ) -> Self {
        Self {
            source_path,
            rust_code,
            dependencies,
            functions_exported,
            ffi_declarations,
        }
    }
}

/// Context for tracking cross-file information during transpilation.
///
/// Maintains knowledge of types, functions, and other declarations
/// across multiple C files to enable proper reference resolution.
#[derive(Debug, Clone, Default)]
pub struct ProjectContext {
    /// Types (structs, enums) defined across the project
    types: HashMap<String, String>,

    /// Functions defined across the project
    functions: HashMap<String, String>,

    /// Transpiled files tracked in this context
    transpiled_files: HashMap<PathBuf, TranspiledFile>,
}

impl ProjectContext {
    /// Create a new empty project context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a transpiled file to the context.
    ///
    /// This makes the file's types and functions available for
    /// cross-file reference resolution.
    pub fn add_transpiled_file(&mut self, file: &TranspiledFile) {
        // Track file
        self.transpiled_files
            .insert(file.source_path.clone(), file.clone());

        // Extract types from rust_code (simplified: just track that types exist)
        // In real implementation, would parse the Rust code
        if file.rust_code.contains("struct") {
            // Extract struct names (simplified pattern matching)
            for line in file.rust_code.lines() {
                if line.contains("struct") {
                    if let Some(name) = self.extract_type_name(line) {
                        self.types.insert(name.clone(), line.to_string());
                    }
                }
            }
        }

        // Track exported functions
        for func_name in &file.functions_exported {
            self.functions.insert(
                func_name.clone(),
                file.source_path.to_string_lossy().to_string(),
            );
        }
    }

    /// Check if a type is defined in the project context.
    pub fn has_type(&self, type_name: &str) -> bool {
        self.types.contains_key(type_name)
    }

    /// Check if a function is defined in the project context.
    pub fn has_function(&self, func_name: &str) -> bool {
        self.functions.contains_key(func_name)
    }

    /// Get the source file that defines a given function.
    pub fn get_function_source(&self, func_name: &str) -> Option<&str> {
        self.functions.get(func_name).map(|s| s.as_str())
    }

    /// Helper: Extract type name from a line containing struct/enum definition
    fn extract_type_name(&self, line: &str) -> Option<String> {
        // Simplified: Extract "Point" from "pub struct Point {"
        let words: Vec<&str> = line.split_whitespace().collect();
        if let Some(idx) = words.iter().position(|&w| w == "struct" || w == "enum") {
            if idx + 1 < words.len() {
                let name = words[idx + 1].trim_end_matches('{').trim_end_matches('<');
                return Some(name.to_string());
            }
        }
        None
    }
}

/// Dependency graph for tracking file dependencies and computing build order.
///
/// Uses a directed acyclic graph (DAG) to represent file dependencies,
/// where an edge from A to B means "A depends on B" (A includes B).
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Directed graph where nodes are file paths
    graph: DiGraph<PathBuf, ()>,

    /// Map from file path to node index for fast lookups
    path_to_node: HashMap<PathBuf, NodeIndex>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph.
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            path_to_node: HashMap::new(),
        }
    }

    /// Check if the graph is empty (has no files).
    pub fn is_empty(&self) -> bool {
        self.graph.node_count() == 0
    }

    /// Get the number of files in the graph.
    pub fn file_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Check if a file is in the graph.
    pub fn contains_file(&self, path: &Path) -> bool {
        self.path_to_node.contains_key(path)
    }

    /// Add a file to the graph.
    ///
    /// If the file already exists, this is a no-op.
    pub fn add_file(&mut self, path: &Path) {
        if !self.contains_file(path) {
            let node = self.graph.add_node(path.to_path_buf());
            self.path_to_node.insert(path.to_path_buf(), node);
        }
    }

    /// Add a dependency relationship: `from` depends on `to`.
    ///
    /// Both files must already be added to the graph via `add_file`.
    pub fn add_dependency(&mut self, from: &Path, to: &Path) {
        let from_node = *self
            .path_to_node
            .get(from)
            .expect("from file must be added to graph first");
        let to_node = *self
            .path_to_node
            .get(to)
            .expect("to file must be added to graph first");

        self.graph.add_edge(from_node, to_node, ());
    }

    /// Check if there is a direct dependency from `from` to `to`.
    pub fn has_dependency(&self, from: &Path, to: &Path) -> bool {
        if let (Some(&from_node), Some(&to_node)) =
            (self.path_to_node.get(from), self.path_to_node.get(to))
        {
            self.graph.contains_edge(from_node, to_node)
        } else {
            false
        }
    }

    /// Compute topological sort to determine build order.
    ///
    /// Returns files in the order they should be transpiled (dependencies first).
    /// Returns an error if there are circular dependencies.
    pub fn topological_sort(&self) -> Result<Vec<PathBuf>> {
        // Check for cycles first
        if petgraph::algo::is_cyclic_directed(&self.graph) {
            return Err(anyhow::anyhow!(
                "Circular dependency detected in file dependencies"
            ));
        }

        let mut topo = Topo::new(&self.graph);
        let mut build_order = Vec::new();

        while let Some(node) = topo.next(&self.graph) {
            if let Some(path) = self.graph.node_weight(node) {
                build_order.push(path.clone());
            }
        }

        // Reverse because we want dependencies before dependents
        build_order.reverse();

        Ok(build_order)
    }

    /// Build a dependency graph from a list of C files.
    ///
    /// Parses #include directives to build the dependency graph.
    pub fn from_files(files: &[PathBuf]) -> Result<Self> {
        let mut graph = Self::new();

        // Add all files first
        for file in files {
            graph.add_file(file);
        }

        // Parse dependencies
        for file in files {
            let content = std::fs::read_to_string(file)
                .with_context(|| format!("Failed to read file: {}", file.display()))?;

            let includes = Self::parse_include_directives(&content);

            // Resolve #include paths relative to the file's directory
            let file_dir = file.parent().unwrap_or_else(|| Path::new("."));

            for include in includes {
                let include_path = file_dir.join(&include);

                // Only add dependency if the included file is in our file list
                if graph.contains_file(&include_path) {
                    graph.add_dependency(file, &include_path);
                }
            }
        }

        Ok(graph)
    }

    /// Parse #include directives from C source code.
    ///
    /// Returns a list of filenames (e.g., ["utils.h", "stdio.h"]).
    pub fn parse_include_directives(code: &str) -> Vec<String> {
        let mut includes = Vec::new();

        for line in code.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("#include") {
                // Extract filename from #include "file.h" or #include <file.h>
                if let Some(start) = trimmed.find('"').or_else(|| trimmed.find('<')) {
                    let end_char = if trimmed.chars().nth(start) == Some('"') {
                        '"'
                    } else {
                        '>'
                    };
                    if let Some(end) = trimmed[start + 1..].find(end_char) {
                        let filename = &trimmed[start + 1..start + 1 + end];
                        includes.push(filename.to_string());
                    }
                }
            }
        }

        includes
    }

    /// Check if a C header file has header guards (#ifndef/#define/#endif).
    pub fn has_header_guard(path: &Path) -> Result<bool> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let has_ifndef = content.lines().any(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("#ifndef") || trimmed.starts_with("#if !defined")
        });

        let has_define = content
            .lines()
            .any(|line| line.trim().starts_with("#define"));
        let has_endif = content
            .lines()
            .any(|line| line.trim().starts_with("#endif"));

        Ok(has_ifndef && has_define && has_endif)
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Main transpilation pipeline entry point.
///
/// Converts C source code to safe Rust code with automatic ownership
/// and lifetime inference.
///
/// # Examples
///
/// ```no_run
/// use decy_core::transpile;
///
/// let c_code = "int add(int a, int b) { return a + b; }";
/// let rust_code = transpile(c_code)?;
/// assert!(rust_code.contains("fn add"));
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - C code parsing fails
/// - HIR conversion fails
/// - Code generation fails
pub fn transpile(c_code: &str) -> Result<String> {
    // Step 1: Parse C code
    let parser = CParser::new().context("Failed to create C parser")?;
    let ast = parser.parse(c_code).context("Failed to parse C code")?;

    // Step 2: Convert to HIR
    let hir_functions: Vec<HirFunction> = ast
        .functions()
        .iter()
        .map(HirFunction::from_ast_function)
        .collect();

    // Convert structs to HIR
    let hir_structs: Vec<decy_hir::HirStruct> = ast
        .structs()
        .iter()
        .map(|s| {
            let fields = s
                .fields
                .iter()
                .map(|f| {
                    decy_hir::HirStructField::new(
                        f.name.clone(),
                        decy_hir::HirType::from_ast_type(&f.field_type),
                    )
                })
                .collect();
            decy_hir::HirStruct::new(s.name.clone(), fields)
        })
        .collect();

    // Step 3: Analyze ownership and lifetimes
    let mut transformed_functions = Vec::new();

    for func in hir_functions {
        // Build dataflow graph for the function
        let dataflow_analyzer = DataflowAnalyzer::new();
        let dataflow_graph = dataflow_analyzer.analyze(&func);

        // Infer ownership patterns
        let ownership_inferencer = OwnershipInferencer::new();
        let ownership_inferences = ownership_inferencer.infer(&dataflow_graph);

        // Generate borrow code (&T, &mut T)
        let borrow_generator = BorrowGenerator::new();
        let func_with_borrows = borrow_generator.transform_function(&func, &ownership_inferences);

        // Analyze lifetimes
        let lifetime_analyzer = LifetimeAnalyzer::new();
        let scope_tree = lifetime_analyzer.build_scope_tree(&func_with_borrows);
        let _lifetimes = lifetime_analyzer.track_lifetimes(&func_with_borrows, &scope_tree);

        // Generate lifetime annotations
        let lifetime_annotator = LifetimeAnnotator::new();
        let annotated_signature = lifetime_annotator.annotate_function(&func_with_borrows);

        // Store both function and its annotated signature
        transformed_functions.push((func_with_borrows, annotated_signature));
    }

    // Step 4: Generate Rust code with lifetime annotations
    let code_generator = CodeGenerator::new();
    let mut rust_code = String::new();

    // Generate struct definitions first
    for hir_struct in &hir_structs {
        let struct_code = code_generator.generate_struct(hir_struct);
        rust_code.push_str(&struct_code);
        rust_code.push('\n');
    }

    // Generate functions with struct definitions for field type awareness
    for (func, annotated_sig) in &transformed_functions {
        let generated = code_generator.generate_function_with_lifetimes_and_structs(
            func,
            annotated_sig,
            &hir_structs,
        );
        rust_code.push_str(&generated);
        rust_code.push('\n');
    }

    Ok(rust_code)
}

/// Transpile with Box transformation enabled.
///
/// This variant applies Box pattern detection to transform malloc/free
/// patterns into safe Box allocations.
///
/// # Examples
///
/// ```no_run
/// use decy_core::transpile_with_box_transform;
///
/// let c_code = r#"
///     int* create_value() {
///         int* p = malloc(sizeof(int));
///         *p = 42;
///         return p;
///     }
/// "#;
/// let rust_code = transpile_with_box_transform(c_code)?;
/// assert!(rust_code.contains("Box"));
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn transpile_with_box_transform(c_code: &str) -> Result<String> {
    // Step 1: Parse C code
    let parser = CParser::new().context("Failed to create C parser")?;
    let ast = parser.parse(c_code).context("Failed to parse C code")?;

    // Step 2: Convert to HIR
    let hir_functions: Vec<HirFunction> = ast
        .functions()
        .iter()
        .map(HirFunction::from_ast_function)
        .collect();

    // Step 3: Generate Rust code with Box transformation
    let code_generator = CodeGenerator::new();
    let pattern_detector = PatternDetector::new();
    let mut rust_code = String::new();

    for func in &hir_functions {
        // Detect Box candidates in this function
        let candidates = pattern_detector.find_box_candidates(func);

        let generated = code_generator.generate_function_with_box_transform(func, &candidates);
        rust_code.push_str(&generated);
        rust_code.push('\n');
    }

    Ok(rust_code)
}

/// Transpile a single C file with project context.
///
/// This enables file-by-file transpilation for incremental C→Rust migration.
/// The `ProjectContext` tracks types and functions across files for proper
/// reference resolution.
///
/// # Examples
///
/// ```no_run
/// use decy_core::{transpile_file, ProjectContext};
/// use std::path::Path;
///
/// let path = Path::new("src/utils.c");
/// let context = ProjectContext::new();
/// let result = transpile_file(path, &context)?;
///
/// assert!(!result.rust_code.is_empty());
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - File does not exist or cannot be read
/// - C code parsing fails
/// - Code generation fails
pub fn transpile_file(path: &Path, _context: &ProjectContext) -> Result<TranspiledFile> {
    // Read the C source file
    let c_code = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    // Parse dependencies from #include directives (simplified: just detect presence)
    let dependencies = extract_dependencies(path, &c_code)?;

    // Transpile the C code using the main pipeline
    let rust_code = transpile(&c_code)?;

    // Extract function names from the generated Rust code
    let functions_exported = extract_function_names(&rust_code);

    // Generate FFI declarations for exported functions
    let ffi_declarations = generate_ffi_declarations(&functions_exported);

    Ok(TranspiledFile::new(
        path.to_path_buf(),
        rust_code,
        dependencies,
        functions_exported,
        ffi_declarations,
    ))
}

/// Extract dependencies from #include directives in C code.
///
/// This is a simplified implementation that detects #include directives
/// and resolves them relative to the source file's directory.
fn extract_dependencies(source_path: &Path, c_code: &str) -> Result<Vec<PathBuf>> {
    let mut dependencies = Vec::new();
    let source_dir = source_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Source file has no parent directory"))?;

    for line in c_code.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#include") {
            // Extract header filename from #include "header.h" or #include <header.h>
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    let header_name = &trimmed[start + 1..start + 1 + end];
                    let header_path = source_dir.join(header_name);
                    if header_path.exists() {
                        dependencies.push(header_path);
                    }
                }
            }
        }
    }

    Ok(dependencies)
}

/// Extract function names from generated Rust code.
///
/// Parses function definitions to identify exported functions.
fn extract_function_names(rust_code: &str) -> Vec<String> {
    let mut functions = Vec::new();

    for line in rust_code.lines() {
        let trimmed = line.trim();
        // Look for "fn function_name(" or "pub fn function_name("
        if (trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ")) && trimmed.contains('(') {
            let start_idx = if trimmed.starts_with("pub fn ") {
                7 // length of "pub fn "
            } else {
                3 // length of "fn "
            };

            if let Some(paren_idx) = trimmed[start_idx..].find('(') {
                let func_name = &trimmed[start_idx..start_idx + paren_idx];
                // Remove generic parameters if present (e.g., "foo<'a>" → "foo")
                let func_name_clean = if let Some(angle_idx) = func_name.find('<') {
                    &func_name[..angle_idx]
                } else {
                    func_name
                };
                functions.push(func_name_clean.trim().to_string());
            }
        }
    }

    functions
}

/// Generate FFI declarations for exported functions.
///
/// Creates extern "C" declarations to enable C↔Rust interoperability.
fn generate_ffi_declarations(functions: &[String]) -> String {
    if functions.is_empty() {
        return String::new();
    }

    let mut ffi = String::from("// FFI declarations for C interoperability\n");
    ffi.push_str("#[no_mangle]\n");
    ffi.push_str("extern \"C\" {\n");

    for func_name in functions {
        ffi.push_str(&format!("    // {}\n", func_name));
    }

    ffi.push_str("}\n");
    ffi
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(
            result.is_ok(),
            "Transpilation with lifetime analysis should succeed"
        );

        let rust_code = result.unwrap();
        // Basic transpilation should work
        assert!(rust_code.contains("fn add"));

        // When references are present, lifetime annotations would appear
        // Future: Add a test with actual C pointer parameters to verify '<'a> syntax
    }
}
