//! HIR visualization for debugging
//!
//! Display High-level IR conversion from C AST

use anyhow::{Context, Result};
use colored::Colorize;
use decy_hir::HirFunction;
use decy_parser::CParser;
use std::fs;
use std::path::Path;

/// Visualize HIR conversion from C source
///
/// # Errors
///
/// Returns an error if the file cannot be read, parsed, or converted to HIR
pub fn visualize_hir(file_path: &Path, use_colors: bool) -> Result<String> {
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

    // Format output
    let mut output = String::new();

    // Header
    if use_colors {
        output.push_str(&format!(
            "{}\n",
            "╔═══ HIR Visualization ═══╗".green().bold()
        ));
    } else {
        output.push_str("╔═══ HIR Visualization ═══╗\n");
    }
    output.push('\n');

    // File info
    output.push_str(&format!("File: {}\n", file_path.display()));
    output.push_str(&format!("HIR Functions: {}\n\n", hir_functions.len()));

    // Display each HIR function
    for hir_func in &hir_functions {
        if use_colors {
            output.push_str(&format!("{}:\n", hir_func.name().bright_cyan().bold()));
        } else {
            output.push_str(&format!("{}:\n", hir_func.name()));
        }

        output.push_str(&format!("  Return type: {:?}\n", hir_func.return_type()));
        output.push_str(&format!("  Parameters: {}\n", hir_func.parameters().len()));

        // Show parameter details
        for param in hir_func.parameters() {
            output.push_str(&format!(
                "    - {} : {:?}\n",
                param.name(),
                param.param_type()
            ));
        }

        output.push_str(&format!("  Body statements: {}\n", hir_func.body().len()));
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
    fn test_visualize_hir_simple() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "int add(int a, int b) {{ return a + b; }}").unwrap();

        let result = visualize_hir(temp_file.path(), false);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("add"));
        assert!(output.contains("HIR Functions"));
    }
}
