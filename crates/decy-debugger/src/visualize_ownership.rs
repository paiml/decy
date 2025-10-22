//! Ownership graph visualization
//!
//! Display ownership inference and dataflow analysis results

use anyhow::{Context, Result};
use colored::Colorize;
use decy_hir::HirFunction;
use decy_parser::CParser;
use std::fs;
use std::path::Path;

/// Visualize ownership inference graph
///
/// # Errors
///
/// Returns an error if the file cannot be analyzed
pub fn visualize_ownership_graph(file_path: &Path, use_colors: bool) -> Result<String> {
    // Read and parse source
    let source = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    let parser = CParser::new()?;
    let ast = parser.parse(&source).context("Failed to parse C source")?;

    // Convert to HIR
    let hir_functions: Vec<HirFunction> = ast
        .functions()
        .iter()
        .map(HirFunction::from_ast_function)
        .collect();

    // Generate output
    let mut output = String::new();

    // Header
    if use_colors {
        output.push_str(&format!(
            "{}\n",
            "╔═══ Ownership Analysis ═══╗".magenta().bold()
        ));
    } else {
        output.push_str("╔═══ Ownership Analysis ═══╗\n");
    }
    output.push('\n');

    output.push_str(&format!("File: {}\n", file_path.display()));
    output.push_str(&format!("Functions analyzed: {}\n\n", hir_functions.len()));

    // Analyze each function
    for hir_func in &hir_functions {
        if use_colors {
            output.push_str(&format!("Function: {}\n", hir_func.name().bright_cyan()));
        } else {
            output.push_str(&format!("Function: {}\n", hir_func.name()));
        }

        // NOTE: Full ownership inference integration deferred to Sprint 18
        // The ownership inference API requires DataflowGraph construction
        // which is a multi-step process. See roadmap.yaml DECY-055 for details.
        output.push_str("  [Ownership analysis: Sprint 18+]\n");
        output.push_str(&format!("  Parameters: {}\n", hir_func.parameters().len()));
        output.push('\n');
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_visualize_ownership_simple() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "int add(int a, int b) {{ return a + b; }}").unwrap();

        let result = visualize_ownership_graph(temp_file.path(), false);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("Ownership Analysis"));
    }
}
