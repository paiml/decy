//! Interactive step-through debugger
//!
//! Step through the transpilation pipeline interactively

use std::path::Path;

/// Run interactive step-through debugging
pub fn interactive_step_through(file_path: &Path, _verbose: bool) {
    println!("═══ Interactive Step-Through Debugger ═══");
    println!("File: {}", file_path.display());
    println!();
    println!("Note: Interactive mode coming in future release.");
    println!("For now, use:");
    println!("  decy debug --visualize-ast <file>");
    println!("  decy debug --visualize-hir <file>");
    println!("  decy debug --visualize-ownership <file>");
}
