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

pub mod metrics;
pub mod optimize;
pub mod trace;

pub use metrics::{
    CompileMetrics, ConvergenceReport, EquivalenceMetrics, TierMetrics, TranspilationResult,
};

use anyhow::{Context, Result};
use decy_analyzer::patterns::PatternDetector;
use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirStatement};
use decy_ownership::{
    array_slice::ArrayParameterTransformer, borrow_gen::BorrowGenerator,
    classifier_integration::classify_with_rules, dataflow::DataflowAnalyzer,
    lifetime::LifetimeAnalyzer, lifetime_gen::LifetimeAnnotator,
};
use decy_parser::parser::CParser;
use decy_stdlib::StdlibPrototypes;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::Topo;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Result of transpiling a single C file.
///
/// Contains the transpiled Rust code along with metadata about
/// dependencies and exported symbols for cross-file reference tracking.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

/// Statistics for transpilation cache performance.
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    /// Number of cache hits
    pub hits: usize,
    /// Number of cache misses
    pub misses: usize,
    /// Total number of files in cache
    pub total_files: usize,
}

/// Cache entry storing file hash and transpilation result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    /// SHA-256 hash of the file content
    hash: String,
    /// Cached transpilation result
    transpiled: TranspiledFile,
    /// Hashes of dependencies (for invalidation)
    dependency_hashes: HashMap<PathBuf, String>,
}

/// Transpilation cache for avoiding re-transpilation of unchanged files.
///
/// Uses SHA-256 hashing to detect file changes and supports disk persistence.
/// Provides 10-20x speedup on cache hits.
///
/// # Examples
///
/// ```no_run
/// use decy_core::{TranspilationCache, ProjectContext, transpile_file};
/// use std::path::Path;
///
/// let mut cache = TranspilationCache::new();
/// let path = Path::new("src/main.c");
/// let context = ProjectContext::new();
///
/// // First transpilation - cache miss
/// let result = transpile_file(path, &context)?;
/// cache.insert(path, &result);
///
/// // Second access - cache hit (if file unchanged)
/// if let Some(cached) = cache.get(path) {
///     println!("Cache hit! Using cached result");
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct TranspilationCache {
    /// Cache entries mapped by file path
    entries: HashMap<PathBuf, CacheEntry>,
    /// Cache directory for disk persistence
    cache_dir: Option<PathBuf>,
    /// Performance statistics
    hits: usize,
    misses: usize,
}

impl TranspilationCache {
    /// Create a new empty transpilation cache.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            cache_dir: None,
            hits: 0,
            misses: 0,
        }
    }

    /// Create a cache with a specific directory for persistence.
    pub fn with_directory(cache_dir: &Path) -> Self {
        Self {
            entries: HashMap::new(),
            cache_dir: Some(cache_dir.to_path_buf()),
            hits: 0,
            misses: 0,
        }
    }

    /// Compute SHA-256 hash of a file's content.
    ///
    /// Returns a 64-character hex string.
    pub fn compute_hash(&self, path: &Path) -> Result<String> {
        use sha2::{Digest, Sha256};

        let content = std::fs::read(path)
            .with_context(|| format!("Failed to read file for hashing: {}", path.display()))?;

        let mut hasher = Sha256::new();
        hasher.update(&content);
        let result = hasher.finalize();

        Ok(format!("{:x}", result))
    }

    /// Insert a transpiled file into the cache.
    pub fn insert(&mut self, path: &Path, transpiled: &TranspiledFile) {
        let hash = match self.compute_hash(path) {
            Ok(h) => h,
            Err(_) => return, // Skip caching if hash fails
        };

        // Compute dependency hashes
        let mut dependency_hashes = HashMap::new();
        for dep_path in &transpiled.dependencies {
            if let Ok(dep_hash) = self.compute_hash(dep_path) {
                dependency_hashes.insert(dep_path.clone(), dep_hash);
            }
        }

        let entry = CacheEntry {
            hash,
            transpiled: transpiled.clone(),
            dependency_hashes,
        };

        self.entries.insert(path.to_path_buf(), entry);
    }

    /// Get a cached transpilation result if the file hasn't changed.
    ///
    /// Returns `None` if:
    /// - File is not in cache
    /// - File content has changed
    /// - Any dependency has changed
    pub fn get(&mut self, path: &Path) -> Option<&TranspiledFile> {
        let entry = self.entries.get(&path.to_path_buf())?;

        // Check if file hash matches
        let current_hash = self.compute_hash(path).ok()?;
        if current_hash != entry.hash {
            self.misses += 1;
            return None;
        }

        // Check if any dependency has changed
        for (dep_path, cached_hash) in &entry.dependency_hashes {
            if let Ok(current_dep_hash) = self.compute_hash(dep_path) {
                if &current_dep_hash != cached_hash {
                    self.misses += 1;
                    return None;
                }
            }
        }

        self.hits += 1;
        Some(&entry.transpiled)
    }

    /// Save the cache to disk (if cache_dir is set).
    pub fn save(&self) -> Result<()> {
        let cache_dir = self
            .cache_dir
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Cache directory not set"))?;

        std::fs::create_dir_all(cache_dir).with_context(|| {
            format!("Failed to create cache directory: {}", cache_dir.display())
        })?;

        let cache_file = cache_dir.join("cache.json");
        let json =
            serde_json::to_string_pretty(&self.entries).context("Failed to serialize cache")?;

        std::fs::write(&cache_file, json)
            .with_context(|| format!("Failed to write cache file: {}", cache_file.display()))?;

        Ok(())
    }

    /// Load a cache from disk.
    pub fn load(cache_dir: &Path) -> Result<Self> {
        let cache_file = cache_dir.join("cache.json");

        if !cache_file.exists() {
            // No cache file exists yet, return empty cache
            return Ok(Self::with_directory(cache_dir));
        }

        let json = std::fs::read_to_string(&cache_file)
            .with_context(|| format!("Failed to read cache file: {}", cache_file.display()))?;

        let entries: HashMap<PathBuf, CacheEntry> =
            serde_json::from_str(&json).context("Failed to deserialize cache")?;

        Ok(Self {
            entries,
            cache_dir: Some(cache_dir.to_path_buf()),
            hits: 0,
            misses: 0,
        })
    }

    /// Clear all cached entries.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.hits = 0;
        self.misses = 0;
    }

    /// Get cache statistics.
    pub fn statistics(&self) -> CacheStatistics {
        CacheStatistics {
            hits: self.hits,
            misses: self.misses,
            total_files: self.entries.len(),
        }
    }
}

impl Default for TranspilationCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Preprocess #include directives in C source code (DECY-056).
///
/// Resolves and inlines #include directives recursively, tracking processed files
/// to prevent infinite loops from circular dependencies.
///
/// **NEW (Stdlib Support)**: Injects built-in C stdlib prototypes when system
/// headers are encountered, enabling parsing of code that uses stdlib functions
/// without requiring actual header files.
///
/// # Arguments
///
/// * `source` - C source code with #include directives
/// * `base_dir` - Base directory for resolving relative include paths (None = current dir)
/// * `processed` - Set of already processed file paths (prevents circular includes)
/// * `stdlib_prototypes` - Stdlib prototype database for injection (None = create new)
/// * `injected_headers` - Set of already injected system headers (prevents duplicates)
///
/// # Returns
///
/// Preprocessed C code with includes inlined and stdlib prototypes injected
fn preprocess_includes(
    source: &str,
    base_dir: Option<&Path>,
    processed: &mut std::collections::HashSet<PathBuf>,
    stdlib_prototypes: &StdlibPrototypes,
    injected_headers: &mut std::collections::HashSet<String>,
) -> Result<String> {
    let mut result = String::new();
    let base_dir = base_dir.unwrap_or_else(|| Path::new("."));

    for line in source.lines() {
        let trimmed = line.trim();

        // Check for #include directive
        if trimmed.starts_with("#include") {
            // Extract filename from #include "file.h" or #include <file.h>
            let (filename, is_system) = if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed[start + 1..].find('"') {
                    let filename = &trimmed[start + 1..start + 1 + end];
                    (filename, false)
                } else {
                    // Malformed include, keep original line
                    result.push_str(line);
                    result.push('\n');
                    continue;
                }
            } else if let Some(start) = trimmed.find('<') {
                if let Some(end) = trimmed[start + 1..].find('>') {
                    let filename = &trimmed[start + 1..start + 1 + end];
                    (filename, true)
                } else {
                    // Malformed include, keep original line
                    result.push_str(line);
                    result.push('\n');
                    continue;
                }
            } else {
                // No include found, keep original line
                result.push_str(line);
                result.push('\n');
                continue;
            };

            // Skip system includes (<stdio.h>) - we don't have those files
            // BUT: Inject stdlib prototypes so parsing succeeds
            if is_system {
                // Comment out the original include
                result.push_str(&format!("// {}\n", line));

                // Inject stdlib prototypes for this header (only once per header)
                if !injected_headers.contains(filename) {
                    // Mark as injected
                    injected_headers.insert(filename.to_string());

                    // Try to parse the header name and inject specific prototypes
                    if let Some(header) = decy_stdlib::StdHeader::from_filename(filename) {
                        result
                            .push_str(&format!("// BEGIN: Built-in prototypes for {}\n", filename));
                        result.push_str(&stdlib_prototypes.inject_prototypes_for_header(header));
                        result.push_str(&format!("// END: Built-in prototypes for {}\n", filename));
                    } else {
                        // Unknown header - just comment it out
                        result.push_str(&format!("// Unknown system header: {}\n", filename));
                    }
                }

                continue;
            }

            // Resolve include path relative to base_dir
            let include_path = base_dir.join(filename);

            // Check if already processed (circular dependency or duplicate)
            if processed.contains(&include_path) {
                // Already processed, skip (header guards working)
                result.push_str(&format!("// Already included: {}\n", filename));
                continue;
            }

            // Try to read the included file
            if let Ok(included_content) = std::fs::read_to_string(&include_path) {
                // Mark as processed
                processed.insert(include_path.clone());

                // Get directory of included file for nested includes
                let included_dir = include_path.parent().unwrap_or(base_dir);

                // Recursively preprocess the included file
                let preprocessed = preprocess_includes(
                    &included_content,
                    Some(included_dir),
                    processed,
                    stdlib_prototypes,
                    injected_headers,
                )?;

                // Add marker comments for debugging
                result.push_str(&format!("// BEGIN INCLUDE: {}\n", filename));
                result.push_str(&preprocessed);
                result.push_str(&format!("// END INCLUDE: {}\n", filename));
            } else {
                // File not found - return error for local includes
                anyhow::bail!("Failed to find include file: {}", include_path.display());
            }
        } else {
            // Regular line, keep as-is
            result.push_str(line);
            result.push('\n');
        }
    }

    Ok(result)
}

/// Main transpilation pipeline entry point.
///
/// Converts C source code to safe Rust code with automatic ownership
/// and lifetime inference.
///
/// Automatically preprocesses #include directives (DECY-056).
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
/// - #include file not found
/// - C code parsing fails
/// - HIR conversion fails
/// - Code generation fails
pub fn transpile(c_code: &str) -> Result<String> {
    transpile_with_includes(c_code, None)
}

/// DECY-193: Transpile C code with decision tracing.
///
/// Returns both the transpiled Rust code and a trace of decisions made
/// during transpilation (ownership inference, type mapping, etc.).
///
/// # Arguments
///
/// * `c_code` - C source code to transpile
///
/// # Returns
///
/// Returns a tuple of (rust_code, trace_collector).
///
/// # Examples
///
/// ```
/// use decy_core::transpile_with_trace;
///
/// let c_code = "int add(int a, int b) { return a + b; }";
/// let (code, trace) = transpile_with_trace(c_code)?;
/// assert!(!code.is_empty());
/// // Trace contains ownership inference decisions
/// let json = trace.to_json();
/// assert!(json.starts_with('['));
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn transpile_with_trace(c_code: &str) -> Result<(String, trace::TraceCollector)> {
    use trace::{DecisionType, PipelineStage, TraceCollector, TraceEntry};

    let mut collector = TraceCollector::new();

    // Record parsing start
    collector.record(TraceEntry {
        stage: PipelineStage::Parsing,
        source_location: None,
        decision_type: DecisionType::PatternDetection,
        chosen: "clang-sys".to_string(),
        alternatives: vec![],
        confidence: 1.0,
        reason: "Using clang-sys for C parsing".to_string(),
    });

    // Transpile normally
    let rust_code = transpile(c_code)?;

    // Record completion
    collector.record(TraceEntry {
        stage: PipelineStage::CodeGeneration,
        source_location: None,
        decision_type: DecisionType::PatternDetection,
        chosen: "completed".to_string(),
        alternatives: vec![],
        confidence: 1.0,
        reason: format!(
            "Transpilation produced {} lines of Rust",
            rust_code.lines().count()
        ),
    });

    Ok((rust_code, collector))
}

/// Transpile C code and return verification result.
///
/// This function transpiles C code to Rust and includes metadata about
/// the transpilation for metrics tracking.
///
/// # Arguments
///
/// * `c_code` - C source code to transpile
///
/// # Returns
///
/// Returns a `TranspilationResult` containing the generated Rust code
/// and verification status.
///
/// # Examples
///
/// ```
/// use decy_core::transpile_with_verification;
///
/// let c_code = "int add(int a, int b) { return a + b; }";
/// let result = transpile_with_verification(c_code);
/// assert!(result.is_ok());
/// ```
pub fn transpile_with_verification(c_code: &str) -> Result<TranspilationResult> {
    match transpile(c_code) {
        Ok(rust_code) => Ok(TranspilationResult::success(rust_code)),
        Err(e) => {
            // Return empty code with error
            Ok(TranspilationResult::failure(
                String::new(),
                vec![e.to_string()],
            ))
        }
    }
}

/// Transpile C code with include directive support and custom base directory.
///
/// # Arguments
///
/// * `c_code` - C source code to transpile
/// * `base_dir` - Base directory for resolving #include paths (None = current dir)
///
/// # Examples
///
/// ```no_run
/// use decy_core::transpile_with_includes;
/// use std::path::Path;
///
/// let c_code = "#include \"utils.h\"\nint main() { return 0; }";
/// let rust_code = transpile_with_includes(c_code, Some(Path::new("/tmp/project")))?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn transpile_with_includes(c_code: &str, base_dir: Option<&Path>) -> Result<String> {
    // Step 0: Preprocess #include directives (DECY-056) + Inject stdlib prototypes
    let stdlib_prototypes = StdlibPrototypes::new();
    let mut processed_files = std::collections::HashSet::new();
    let mut injected_headers = std::collections::HashSet::new();
    let preprocessed = preprocess_includes(
        c_code,
        base_dir,
        &mut processed_files,
        &stdlib_prototypes,
        &mut injected_headers,
    )?;

    // Step 1: Parse C code
    // Note: We don't add standard type definitions (size_t, etc.) here because:
    // 1. If code has #include directives, system headers define them
    // 2. If code doesn't have includes and uses size_t, it should typedef it explicitly
    // 3. Adding conflicting typedefs breaks parsing
    let parser = CParser::new().context("Failed to create C parser")?;
    let ast = parser
        .parse(&preprocessed)
        .context("Failed to parse C code")?;

    // Step 2: Convert to HIR
    let all_hir_functions: Vec<HirFunction> = ast
        .functions()
        .iter()
        .map(HirFunction::from_ast_function)
        .collect();

    // DECY-190: Deduplicate functions - when a C file has both a declaration
    // (prototype) and a definition, only keep the definition.
    // This prevents "the name X is defined multiple times" errors in Rust.
    let hir_functions: Vec<HirFunction> = {
        use std::collections::HashMap;
        let mut func_map: HashMap<String, HirFunction> = HashMap::new();

        for func in all_hir_functions {
            let name = func.name().to_string();
            if let Some(existing) = func_map.get(&name) {
                // Keep the one with a body (definition) over the one without (declaration)
                if func.has_body() && !existing.has_body() {
                    func_map.insert(name, func);
                }
                // Otherwise keep existing (either both have bodies, both don't, or existing has body)
            } else {
                func_map.insert(name, func);
            }
        }

        // DECY-260: Sort by name for deterministic output
        let mut funcs: Vec<_> = func_map.into_values().collect();
        funcs.sort_by(|a, b| a.name().cmp(b.name()));
        funcs
    };

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

    // DECY-240: Convert enums to HIR
    let hir_enums: Vec<decy_hir::HirEnum> = ast
        .enums()
        .iter()
        .map(|e| {
            let variants = e
                .variants
                .iter()
                .map(|v| {
                    decy_hir::HirEnumVariant::new(v.name.clone(), v.value.map(|val| val as i32))
                })
                .collect();
            decy_hir::HirEnum::new(e.name.clone(), variants)
        })
        .collect();

    // Convert global variables to HIR (DECY-054)
    // DECY-223: Filter out extern references (they refer to existing globals, not new definitions)
    // Also deduplicate by name (first definition wins)
    let mut seen_globals: std::collections::HashSet<String> = std::collections::HashSet::new();
    let hir_variables: Vec<decy_hir::HirStatement> = ast
        .variables()
        .iter()
        .filter(|v| {
            // Skip extern declarations without initializers (they're references, not definitions)
            // extern int max; → skip (reference)
            // int max = 0; → keep (definition)
            // extern int max = 0; → keep (definition with extern linkage)
            if v.is_extern() && v.initializer().is_none() {
                return false;
            }
            // Deduplicate by name
            if seen_globals.contains(v.name()) {
                return false;
            }
            seen_globals.insert(v.name().to_string());
            true
        })
        .map(|v| decy_hir::HirStatement::VariableDeclaration {
            name: v.name().to_string(),
            var_type: decy_hir::HirType::from_ast_type(v.var_type()),
            initializer: v
                .initializer()
                .map(decy_hir::HirExpression::from_ast_expression),
        })
        .collect();

    // Convert typedefs to HIR (DECY-054, DECY-057)
    let hir_typedefs: Vec<decy_hir::HirTypedef> = ast
        .typedefs()
        .iter()
        .map(|t| {
            decy_hir::HirTypedef::new(
                t.name().to_string(),
                decy_hir::HirType::from_ast_type(&t.underlying_type),
            )
        })
        .collect();

    // DECY-116: Build slice function arg mappings BEFORE transformation (while we still have original params)
    let slice_func_args: Vec<(String, Vec<(usize, usize)>)> = hir_functions
        .iter()
        .filter_map(|func| {
            let mut mappings = Vec::new();
            let params = func.parameters();

            for (i, param) in params.iter().enumerate() {
                // Check if this is a pointer param (potential array)
                if matches!(param.param_type(), decy_hir::HirType::Pointer(_)) {
                    // Check if next param is an int with length-like name
                    if i + 1 < params.len() {
                        let next_param = &params[i + 1];
                        if matches!(next_param.param_type(), decy_hir::HirType::Int) {
                            let param_name = next_param.name().to_lowercase();
                            if param_name.contains("len")
                                || param_name.contains("size")
                                || param_name.contains("count")
                                || param_name == "n"
                                || param_name == "num"
                            {
                                mappings.push((i, i + 1));
                            }
                        }
                    }
                }
            }

            if mappings.is_empty() {
                None
            } else {
                Some((func.name().to_string(), mappings))
            }
        })
        .collect();

    // Step 3: Analyze ownership and lifetimes
    let mut transformed_functions = Vec::new();

    for func in hir_functions {
        // Build dataflow graph for the function
        let dataflow_analyzer = DataflowAnalyzer::new();
        let dataflow_graph = dataflow_analyzer.analyze(&func);

        // DECY-183: Infer ownership patterns using RuleBasedClassifier
        // This replaces OwnershipInferencer with the new classifier system
        let ownership_inferences = classify_with_rules(&dataflow_graph, &func);

        // Generate borrow code (&T, &mut T)
        let borrow_generator = BorrowGenerator::new();
        let func_with_borrows = borrow_generator.transform_function(&func, &ownership_inferences);

        // DECY-072 GREEN: Transform array parameters to slices
        let array_transformer = ArrayParameterTransformer::new();
        let func_with_slices = array_transformer.transform(&func_with_borrows, &dataflow_graph);

        // Analyze lifetimes
        let lifetime_analyzer = LifetimeAnalyzer::new();
        let scope_tree = lifetime_analyzer.build_scope_tree(&func_with_slices);
        let _lifetimes = lifetime_analyzer.track_lifetimes(&func_with_slices, &scope_tree);

        // Generate lifetime annotations
        let lifetime_annotator = LifetimeAnnotator::new();
        let annotated_signature = lifetime_annotator.annotate_function(&func_with_slices);

        // DECY-196: Run HIR optimization passes before codegen
        let optimized_func = optimize::optimize_function(&func_with_slices);

        // Store both function and its annotated signature
        transformed_functions.push((optimized_func, annotated_signature));
    }

    // Step 4: Generate Rust code with lifetime annotations
    let code_generator = CodeGenerator::new();
    let mut rust_code = String::new();

    // DECY-119: Track emitted definitions to avoid duplicates
    let mut emitted_structs = std::collections::HashSet::new();
    let mut emitted_typedefs = std::collections::HashSet::new();

    // Generate struct definitions first (deduplicated)
    for hir_struct in &hir_structs {
        let struct_name = hir_struct.name();
        if emitted_structs.contains(struct_name) {
            continue; // Skip duplicate
        }
        emitted_structs.insert(struct_name.to_string());

        let struct_code = code_generator.generate_struct(hir_struct);
        rust_code.push_str(&struct_code);
        rust_code.push('\n');
    }

    // DECY-240: Generate enum definitions (as const i32 values)
    for hir_enum in &hir_enums {
        let enum_code = code_generator.generate_enum(hir_enum);
        rust_code.push_str(&enum_code);
        rust_code.push('\n');
    }

    // DECY-241: Add errno global variable (C compatibility)
    rust_code.push_str("static mut ERRNO: i32 = 0;\n");

    // Generate typedefs (DECY-054, DECY-057) - deduplicated
    for typedef in &hir_typedefs {
        let typedef_name = typedef.name();
        if emitted_typedefs.contains(typedef_name) {
            continue; // Skip duplicate
        }
        emitted_typedefs.insert(typedef_name.to_string());

        if let Ok(typedef_code) = code_generator.generate_typedef(typedef) {
            rust_code.push_str(&typedef_code);
            rust_code.push('\n');
        }
    }

    // DECY-246: Helper to generate const struct literal for static initialization
    // Creates "StructName { field1: default1, field2: default2, ... }" for const contexts
    let const_struct_literal = |struct_name: &str| -> String {
        // Find the struct definition
        if let Some(hir_struct) = hir_structs.iter().find(|s| s.name() == struct_name) {
            let field_inits: Vec<String> = hir_struct
                .fields()
                .iter()
                .map(|f| {
                    let default_val = match f.field_type() {
                        decy_hir::HirType::Int => "0".to_string(),
                        decy_hir::HirType::UnsignedInt => "0".to_string(),
                        decy_hir::HirType::Char => "0".to_string(),
                        decy_hir::HirType::SignedChar => "0".to_string(), // DECY-250
                        decy_hir::HirType::Float => "0.0".to_string(),
                        decy_hir::HirType::Double => "0.0".to_string(),
                        decy_hir::HirType::Pointer(_) => "std::ptr::null_mut()".to_string(),
                        decy_hir::HirType::Array {
                            size: Some(n),
                            element_type,
                        } => {
                            let elem = match element_type.as_ref() {
                                decy_hir::HirType::Char => "0u8",
                                decy_hir::HirType::SignedChar => "0i8", // DECY-250
                                decy_hir::HirType::Int => "0i32",
                                _ => "0",
                            };
                            format!("[{}; {}]", elem, n)
                        }
                        _ => "Default::default()".to_string(),
                    };
                    format!("{}: {}", f.name(), default_val)
                })
                .collect();
            format!("{} {{ {} }}", struct_name, field_inits.join(", "))
        } else {
            // Fallback if struct not found
            format!("{}::default()", struct_name)
        }
    };

    // Generate global variables (DECY-054)
    // DECY-220: Collect global variable names for unsafe access tracking
    // DECY-233: Also collect types for proper type inference in function bodies
    let mut global_vars: Vec<(String, decy_hir::HirType)> = Vec::new();
    for var_stmt in &hir_variables {
        if let decy_hir::HirStatement::VariableDeclaration {
            name,
            var_type,
            initializer,
        } = var_stmt
        {
            // DECY-220/233: Track global name and type for unsafe access and type inference
            global_vars.push((name.clone(), var_type.clone()));
            // Generate as static mut for C global variable equivalence
            let type_str = CodeGenerator::map_type(var_type);

            if let Some(init_expr) = initializer {
                // DECY-201: Special handling for array initialization
                // DECY-246: Handle struct arrays using StructName::default()
                let init_code = if let decy_hir::HirType::Array {
                    element_type,
                    size: Some(size_val),
                } = var_type
                {
                    // Check if init_expr is just an integer (uninitialized or zero-initialized array)
                    if matches!(init_expr, decy_hir::HirExpression::IntLiteral(_)) {
                        // Use type-appropriate const default value (for static context)
                        let element_init = match element_type.as_ref() {
                            decy_hir::HirType::Char => "0u8".to_string(),
                            decy_hir::HirType::SignedChar => "0i8".to_string(), // DECY-250
                            decy_hir::HirType::Int => "0i32".to_string(),
                            decy_hir::HirType::UnsignedInt => "0u32".to_string(),
                            decy_hir::HirType::Float => "0.0f32".to_string(),
                            decy_hir::HirType::Double => "0.0f64".to_string(),
                            decy_hir::HirType::Pointer(_) => "std::ptr::null_mut()".to_string(),
                            // DECY-246: Use const struct literal for statics
                            decy_hir::HirType::Struct(name) => const_struct_literal(name),
                            _ => "0".to_string(),
                        };
                        format!("[{}; {}]", element_init, size_val)
                    } else {
                        code_generator.generate_expression(init_expr)
                    }
                } else {
                    code_generator.generate_expression(init_expr)
                };
                rust_code.push_str(&format!(
                    "static mut {}: {} = {};\n",
                    name, type_str, init_code
                ));
            } else {
                // DECY-215: Use appropriate default values for uninitialized globals
                // Only use Option for function pointers and complex types
                let default_value = match var_type {
                    decy_hir::HirType::Int => "0".to_string(),
                    decy_hir::HirType::UnsignedInt => "0".to_string(),
                    decy_hir::HirType::Char => "0".to_string(),
                    decy_hir::HirType::SignedChar => "0".to_string(), // DECY-250
                    decy_hir::HirType::Float => "0.0".to_string(),
                    decy_hir::HirType::Double => "0.0".to_string(),
                    decy_hir::HirType::Pointer(_) => "std::ptr::null_mut()".to_string(),
                    // DECY-217: Arrays need explicit zero initialization (Default only works for size <= 32)
                    // DECY-246: Handle struct arrays using const struct literal for statics
                    decy_hir::HirType::Array { element_type, size } => {
                        let elem_default = match element_type.as_ref() {
                            decy_hir::HirType::Char => "0u8".to_string(),
                            decy_hir::HirType::SignedChar => "0i8".to_string(), // DECY-250
                            decy_hir::HirType::Int => "0i32".to_string(),
                            decy_hir::HirType::UnsignedInt => "0u32".to_string(),
                            decy_hir::HirType::Float => "0.0f32".to_string(),
                            decy_hir::HirType::Double => "0.0f64".to_string(),
                            decy_hir::HirType::Pointer(_) => "std::ptr::null_mut()".to_string(),
                            // DECY-246: Use const struct literal for statics
                            decy_hir::HirType::Struct(name) => const_struct_literal(name),
                            _ => "0".to_string(),
                        };
                        if let Some(n) = size {
                            format!("[{}; {}]", elem_default, n)
                        } else {
                            format!("[{}; 0]", elem_default)
                        }
                    }
                    decy_hir::HirType::FunctionPointer { .. } => {
                        // Function pointers need Option wrapping
                        rust_code.push_str(&format!(
                            "static mut {}: Option<{}> = None;\n",
                            name, type_str
                        ));
                        continue;
                    }
                    _ => "Default::default()".to_string(),
                };
                rust_code.push_str(&format!(
                    "static mut {}: {} = {};\n",
                    name, type_str, default_value
                ));
            }
        }
    }
    if !hir_variables.is_empty() {
        rust_code.push('\n');
    }

    // DECY-117: Build function signatures for call site reference mutability
    // DECY-125: Keep pointers as pointers when pointer arithmetic is used
    // DECY-159: Keep pointers as pointers when NULL comparison is used
    let all_function_sigs: Vec<(String, Vec<decy_hir::HirType>)> = transformed_functions
        .iter()
        .map(|(func, _sig)| {
            let param_types: Vec<decy_hir::HirType> = func
                .parameters()
                .iter()
                .map(|p| {
                    // Transform pointer params to mutable references (matching DECY-111)
                    // DECY-125: But keep as pointer if pointer arithmetic is used
                    // DECY-159: Also keep as pointer if compared to NULL (NULL is valid input)
                    if let decy_hir::HirType::Pointer(inner) = p.param_type() {
                        // Check if this param uses pointer arithmetic or is compared to NULL
                        if uses_pointer_arithmetic(func, p.name())
                            || pointer_compared_to_null(func, p.name())
                        {
                            // Keep as raw pointer
                            p.param_type().clone()
                        } else {
                            decy_hir::HirType::Reference {
                                inner: inner.clone(),
                                mutable: true,
                            }
                        }
                    } else {
                        p.param_type().clone()
                    }
                })
                .collect();
            (func.name().to_string(), param_types)
        })
        .collect();

    // DECY-134b: Build string iteration function info for call site transformation
    let string_iter_funcs: Vec<(String, Vec<(usize, bool)>)> = transformed_functions
        .iter()
        .filter_map(|(func, _)| {
            let params = code_generator.get_string_iteration_params(func);
            if params.is_empty() {
                None
            } else {
                Some((func.name().to_string(), params))
            }
        })
        .collect();

    // Generate functions with struct definitions for field type awareness
    // Note: slice_func_args was built at line 814 BEFORE transformation to capture original params
    // DECY-220/233: Pass global_vars for unsafe access tracking and type inference
    for (func, annotated_sig) in &transformed_functions {
        let generated = code_generator.generate_function_with_lifetimes_and_structs(
            func,
            annotated_sig,
            &hir_structs,
            &all_function_sigs,
            &slice_func_args,
            &string_iter_funcs,
            &global_vars,
        );
        rust_code.push_str(&generated);
        rust_code.push('\n');
    }

    Ok(rust_code)
}

/// DECY-237: Transpile directly from a C file path.
/// This uses clang's native file parsing which properly resolves system headers.
///
/// # Arguments
///
/// * `file_path` - Path to the C source file
///
/// # Returns
///
/// Generated Rust code as a string.
pub fn transpile_from_file_path(file_path: &Path) -> Result<String> {
    // Read the source code
    let c_code = std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    // Use transpile_with_file which uses file-based parsing
    transpile_with_file(&c_code, file_path)
}

/// Transpile C code with file-based parsing for proper header resolution.
/// DECY-237: Uses clang's native file parsing instead of in-memory parsing.
fn transpile_with_file(c_code: &str, file_path: &Path) -> Result<String> {
    let base_dir = file_path.parent();

    // Step 0: Preprocess #include directives (DECY-056) + Inject stdlib prototypes
    let stdlib_prototypes = StdlibPrototypes::new();
    let mut processed_files = std::collections::HashSet::new();
    let mut injected_headers = std::collections::HashSet::new();
    let _preprocessed = preprocess_includes(
        c_code,
        base_dir,
        &mut processed_files,
        &stdlib_prototypes,
        &mut injected_headers,
    )?;

    // Step 1: Parse C code using file-based parsing for proper header resolution
    let parser = CParser::new().context("Failed to create C parser")?;
    let ast = parser
        .parse_file(file_path)
        .context("Failed to parse C code")?;

    // The rest is the same as transpile_with_includes
    process_ast_to_rust(ast, base_dir)
}

/// Process an AST into Rust code (shared implementation)
fn process_ast_to_rust(ast: decy_parser::Ast, _base_dir: Option<&Path>) -> Result<String> {
    // Step 2: Convert to HIR
    let all_hir_functions: Vec<HirFunction> = ast
        .functions()
        .iter()
        .map(HirFunction::from_ast_function)
        .collect();

    // DECY-190: Deduplicate functions
    let hir_functions: Vec<HirFunction> = {
        use std::collections::HashMap;
        let mut func_map: HashMap<String, HirFunction> = HashMap::new();

        for func in all_hir_functions {
            let name = func.name().to_string();
            if let Some(existing) = func_map.get(&name) {
                if func.has_body() && !existing.has_body() {
                    func_map.insert(name, func);
                }
            } else {
                func_map.insert(name, func);
            }
        }

        func_map.into_values().collect()
    };

    // Convert structs to HIR
    let hir_structs: Vec<decy_hir::HirStruct> = ast
        .structs()
        .iter()
        .map(|s| {
            let fields = s
                .fields()
                .iter()
                .map(|f| {
                    decy_hir::HirStructField::new(
                        f.name.clone(),
                        decy_hir::HirType::from_ast_type(&f.field_type),
                    )
                })
                .collect();
            decy_hir::HirStruct::new(s.name().to_string(), fields)
        })
        .collect();

    // DECY-240: Convert enums to HIR
    let hir_enums: Vec<decy_hir::HirEnum> = ast
        .enums()
        .iter()
        .map(|e| {
            let variants = e
                .variants
                .iter()
                .map(|v| {
                    decy_hir::HirEnumVariant::new(v.name.clone(), v.value.map(|val| val as i32))
                })
                .collect();
            decy_hir::HirEnum::new(e.name.clone(), variants)
        })
        .collect();

    // Convert global variables with deduplication
    let mut seen_globals = std::collections::HashSet::new();
    let hir_variables: Vec<decy_hir::HirStatement> = ast
        .variables()
        .iter()
        .filter(|v| {
            if seen_globals.contains(v.name()) {
                return false;
            }
            seen_globals.insert(v.name().to_string());
            true
        })
        .map(|v| decy_hir::HirStatement::VariableDeclaration {
            name: v.name().to_string(),
            var_type: decy_hir::HirType::from_ast_type(v.var_type()),
            initializer: v
                .initializer()
                .map(decy_hir::HirExpression::from_ast_expression),
        })
        .collect();

    // Convert typedefs
    let hir_typedefs: Vec<decy_hir::HirTypedef> = ast
        .typedefs()
        .iter()
        .map(|t| {
            decy_hir::HirTypedef::new(
                t.name().to_string(),
                decy_hir::HirType::from_ast_type(&t.underlying_type),
            )
        })
        .collect();

    // Create code generator
    let code_generator = CodeGenerator::new();
    let mut rust_code = String::new();

    // Track emitted items to avoid duplicates
    let mut emitted_structs = std::collections::HashSet::new();
    let mut emitted_typedefs = std::collections::HashSet::new();

    // Generate struct definitions
    for hir_struct in &hir_structs {
        let struct_name = hir_struct.name();
        if emitted_structs.contains(struct_name) {
            continue;
        }
        emitted_structs.insert(struct_name.to_string());
        let struct_code = code_generator.generate_struct(hir_struct);
        rust_code.push_str(&struct_code);
        rust_code.push('\n');
    }

    // DECY-240: Generate enum definitions (as const i32 values)
    for hir_enum in &hir_enums {
        let enum_code = code_generator.generate_enum(hir_enum);
        rust_code.push_str(&enum_code);
        rust_code.push('\n');
    }

    // DECY-241: Add errno global variable (C compatibility)
    rust_code.push_str("static mut ERRNO: i32 = 0;\n");

    // Generate typedefs
    for typedef in &hir_typedefs {
        let typedef_name = typedef.name();
        if emitted_typedefs.contains(typedef_name) {
            continue;
        }
        emitted_typedefs.insert(typedef_name.to_string());
        if let Ok(typedef_code) = code_generator.generate_typedef(typedef) {
            rust_code.push_str(&typedef_code);
            rust_code.push('\n');
        }
    }

    // Generate global variables
    for var_stmt in &hir_variables {
        if let decy_hir::HirStatement::VariableDeclaration {
            name,
            var_type,
            initializer,
        } = var_stmt
        {
            let type_str = CodeGenerator::map_type(var_type);
            if let Some(init_expr) = initializer {
                let init_code = code_generator.generate_expression(init_expr);
                rust_code.push_str(&format!(
                    "static mut {}: {} = {};\n",
                    name, type_str, init_code
                ));
            } else {
                let default_value = match var_type {
                    decy_hir::HirType::Int => "0".to_string(),
                    decy_hir::HirType::UnsignedInt => "0".to_string(),
                    decy_hir::HirType::Char => "0".to_string(),
                    decy_hir::HirType::SignedChar => "0".to_string(), // DECY-250
                    decy_hir::HirType::Float => "0.0".to_string(),
                    decy_hir::HirType::Double => "0.0".to_string(),
                    decy_hir::HirType::Pointer(_) => "std::ptr::null_mut()".to_string(),
                    decy_hir::HirType::Array { element_type, size } => {
                        let elem_default = match element_type.as_ref() {
                            decy_hir::HirType::Char => "0u8",
                            decy_hir::HirType::SignedChar => "0i8", // DECY-250
                            decy_hir::HirType::Int => "0i32",
                            decy_hir::HirType::UnsignedInt => "0u32",
                            decy_hir::HirType::Float => "0.0f32",
                            decy_hir::HirType::Double => "0.0f64",
                            _ => "0",
                        };
                        if let Some(n) = size {
                            format!("[{}; {}]", elem_default, n)
                        } else {
                            format!("[{}; 0]", elem_default)
                        }
                    }
                    _ => "Default::default()".to_string(),
                };
                rust_code.push_str(&format!(
                    "static mut {}: {} = {};\n",
                    name, type_str, default_value
                ));
            }
        }
    }

    // Generate functions
    // DECY-248: Pass structs to enable sizeof(struct_field) type lookup
    for func in &hir_functions {
        rust_code.push_str(&code_generator.generate_function_with_structs(func, &hir_structs));
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

/// Check if a function parameter uses pointer arithmetic (DECY-125).
///
/// Returns true if the parameter is used in `p = p + n` or `p = p - n` patterns.
fn uses_pointer_arithmetic(func: &HirFunction, param_name: &str) -> bool {
    for stmt in func.body() {
        if statement_uses_pointer_arithmetic(stmt, param_name) {
            return true;
        }
    }
    false
}

/// DECY-159: Check if a pointer parameter is compared to NULL.
///
/// If a pointer is compared to NULL, it means NULL is a valid input value.
/// Such parameters must remain as raw pointers, not references,
/// to avoid dereferencing NULL at call sites.
fn pointer_compared_to_null(func: &HirFunction, param_name: &str) -> bool {
    for stmt in func.body() {
        if statement_compares_to_null(stmt, param_name) {
            return true;
        }
    }
    false
}

/// DECY-159: Recursively check if a statement compares a variable to NULL.
fn statement_compares_to_null(stmt: &HirStatement, var_name: &str) -> bool {
    match stmt {
        HirStatement::If {
            condition,
            then_block,
            else_block,
        } => {
            // Check if condition is var == NULL or var != NULL
            if expression_compares_to_null(condition, var_name) {
                return true;
            }
            // Recurse into blocks
            then_block
                .iter()
                .any(|s| statement_compares_to_null(s, var_name))
                || else_block
                    .as_ref()
                    .is_some_and(|blk| blk.iter().any(|s| statement_compares_to_null(s, var_name)))
        }
        HirStatement::While { condition, body } => {
            expression_compares_to_null(condition, var_name)
                || body.iter().any(|s| statement_compares_to_null(s, var_name))
        }
        HirStatement::For {
            condition, body, ..
        } => {
            expression_compares_to_null(condition, var_name)
                || body.iter().any(|s| statement_compares_to_null(s, var_name))
        }
        HirStatement::Switch {
            condition, cases, ..
        } => {
            expression_compares_to_null(condition, var_name)
                || cases.iter().any(|c| {
                    c.body
                        .iter()
                        .any(|s| statement_compares_to_null(s, var_name))
                })
        }
        _ => false,
    }
}

/// DECY-159: Check if an expression compares a variable to NULL.
///
/// NULL in C can be:
/// - `NullLiteral` (explicit NULL)
/// - `IntLiteral(0)` (NULL macro expanded to 0)
/// - Cast of 0 to void* (less common)
fn expression_compares_to_null(expr: &HirExpression, var_name: &str) -> bool {
    use decy_hir::BinaryOperator;
    match expr {
        HirExpression::BinaryOp { op, left, right } => {
            // Check for var == NULL or var != NULL
            if matches!(op, BinaryOperator::Equal | BinaryOperator::NotEqual) {
                let left_is_var =
                    matches!(&**left, HirExpression::Variable(name) if name == var_name);
                let right_is_var =
                    matches!(&**right, HirExpression::Variable(name) if name == var_name);
                // NULL can be NullLiteral or IntLiteral(0) (when NULL macro expands to 0)
                let left_is_null = matches!(
                    &**left,
                    HirExpression::NullLiteral | HirExpression::IntLiteral(0)
                );
                let right_is_null = matches!(
                    &**right,
                    HirExpression::NullLiteral | HirExpression::IntLiteral(0)
                );

                if (left_is_var && right_is_null) || (right_is_var && left_is_null) {
                    return true;
                }
            }
            // Recurse into binary op children for nested comparisons
            expression_compares_to_null(left, var_name)
                || expression_compares_to_null(right, var_name)
        }
        HirExpression::UnaryOp { operand, .. } => expression_compares_to_null(operand, var_name),
        _ => false,
    }
}

/// Recursively check if a statement uses pointer arithmetic on a variable (DECY-125).
fn statement_uses_pointer_arithmetic(stmt: &HirStatement, var_name: &str) -> bool {
    use decy_hir::BinaryOperator;
    match stmt {
        HirStatement::Assignment { target, value } => {
            // Check if this is var = var + n or var = var - n (pointer arithmetic)
            if target == var_name {
                if let HirExpression::BinaryOp { op, left, .. } = value {
                    if matches!(op, BinaryOperator::Add | BinaryOperator::Subtract) {
                        if let HirExpression::Variable(name) = &**left {
                            if name == var_name {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        }
        HirStatement::If {
            then_block,
            else_block,
            ..
        } => {
            then_block
                .iter()
                .any(|s| statement_uses_pointer_arithmetic(s, var_name))
                || else_block.as_ref().is_some_and(|blk| {
                    blk.iter()
                        .any(|s| statement_uses_pointer_arithmetic(s, var_name))
                })
        }
        HirStatement::While { body, .. } | HirStatement::For { body, .. } => body
            .iter()
            .any(|s| statement_uses_pointer_arithmetic(s, var_name)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(
            ctx.extract_type_name("pub struct Point {"),
            Some("Point".to_string())
        );
    }

    #[test]
    fn test_project_context_extract_type_name_enum() {
        let ctx = ProjectContext::new();
        assert_eq!(
            ctx.extract_type_name("pub enum Color {"),
            Some("Color".to_string())
        );
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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circular dependency"));
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

        std::fs::write(
            &main_path,
            "#include \"dep.h\"\nint main() { return foo(); }",
        )
        .unwrap();
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

        let params = vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )];

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

        let params = vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )];

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

        let params = vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )];

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

        let params = vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )];

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

        let params = vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )];

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

        let params = vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )];

        let body = vec![HirStatement::For {
            init: vec![],
            condition: HirExpression::IntLiteral(1), // true condition
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

        let params = vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )];

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
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::NotEqual,
                left: Box::new(HirExpression::Variable("ptr".to_string())),
                right: Box::new(HirExpression::NullLiteral),
            },
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

        let body = vec![HirStatement::Expression(HirExpression::Variable(
            "ptr".to_string(),
        ))];

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
        let expr = HirExpression::FunctionCall {
            function: "foo".to_string(),
            arguments: vec![],
        };
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
            condition: HirExpression::IntLiteral(1), // true condition
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
}
