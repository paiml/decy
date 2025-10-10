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
        let _annotated_signature = lifetime_annotator.annotate_function(&func_with_borrows);

        // For now, use the function with borrows
        // Future: integrate lifetime annotations into code generation
        transformed_functions.push(func_with_borrows);
    }

    // Step 4: Generate Rust code
    let code_generator = CodeGenerator::new();
    let mut rust_code = String::new();

    for func in &transformed_functions {
        let generated = code_generator.generate_function(func);
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
}
