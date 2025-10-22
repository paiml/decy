//! Decy Interactive Debugger
//!
//! Deep integration with spydecy-debugger to provide introspective debugging
//! capabilities for the Decy C-to-Rust transpiler.
//!
//! # Features
//!
//! - **C AST Visualization**: Display parsed C AST with colored tree view
//! - **HIR Visualization**: Show High-level IR conversion from C
//! - **Ownership Graph**: Visualize pointer ownership inference
//! - **Dataflow Graph**: Display dataflow analysis results
//! - **Step-through Debugging**: Step through transpilation pipeline
//! - **Diff Viewer**: Compare input C vs output Rust
//!
//! # Architecture
//!
//! ```text
//! C Source → Parser → AST → HIR → Analyzer → Codegen → Rust
//!              ↓        ↓     ↓       ↓         ↓        ↓
//!           [Debug] [Debug] [Debug] [Debug]  [Debug] [Debug]
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use decy_debugger::DebuGGER;
//! use std::path::Path;
//!
//! # fn main() -> anyhow::Result<()> {
//! let debugger = Debugger::new();
//!
//! // Visualize C AST
//! let ast_output = debugger.visualize_c_ast(Path::new("example.c"))?;
//! println!("{}", ast_output);
//!
//! // Show ownership graph
//! let ownership_output = debugger.visualize_ownership(Path::new("example.c"))?;
//! println!("{}", ownership_output);
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs, clippy::all, clippy::pedantic)]
#![deny(unsafe_code)]
#![allow(
    clippy::module_name_repetitions,
    clippy::format_push_string,
    clippy::str_to_string,
    clippy::unwrap_used
)]

pub mod step_debugger;
pub mod visualize_ast;
pub mod visualize_hir;
pub mod visualize_ownership;

use anyhow::Result;
use std::path::Path;

/// Main debugger interface for Decy transpiler
///
/// Provides introspective debugging capabilities by integrating with
/// spydecy-debugger and adding C-specific visualizations.
#[derive(Debug, Default)]
pub struct Debugger {
    /// Enable verbose output
    pub verbose: bool,
    /// Enable colored output (default: true)
    pub colored: bool,
}

impl Debugger {
    /// Create a new debugger instance
    #[must_use]
    pub fn new() -> Self {
        Self {
            verbose: false,
            colored: true,
        }
    }

    /// Visualize C AST from source file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed
    pub fn visualize_c_ast(&self, file_path: &Path) -> Result<String> {
        visualize_ast::visualize_c_ast(file_path, self.colored)
    }

    /// Visualize HIR (High-level IR) conversion
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, parsed, or converted to HIR
    pub fn visualize_hir(&self, file_path: &Path) -> Result<String> {
        visualize_hir::visualize_hir(file_path, self.colored)
    }

    /// Visualize ownership inference graph
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be analyzed
    pub fn visualize_ownership(&self, file_path: &Path) -> Result<String> {
        visualize_ownership::visualize_ownership_graph(file_path, self.colored)
    }

    /// Step through transpilation pipeline interactively
    ///
    /// # Errors
    ///
    /// Returns an error if the pipeline cannot be initialized
    pub fn step_through(&self, file_path: &Path) -> Result<()> {
        step_debugger::interactive_step_through(file_path, self.verbose)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_debugger_creation() {
        let debugger = Debugger::new();
        assert!(!debugger.verbose);
        assert!(debugger.colored);
    }

    #[test]
    fn test_visualize_simple_c_function() {
        let debugger = Debugger::new();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "int add(int a, int b) {{ return a + b; }}").unwrap();

        let result = debugger.visualize_c_ast(temp_file.path());
        assert!(result.is_ok(), "Should visualize simple C function");

        let output = result.unwrap();
        assert!(output.contains("Function"), "Should contain Function node");
        assert!(output.contains("add"), "Should contain function name");
    }
}
