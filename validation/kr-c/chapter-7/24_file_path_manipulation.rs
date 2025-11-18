/* K&R C Chapter 7: File Path Manipulation
 * Transpiled to safe Rust (using std::path)
 */

use std::path::{Path, PathBuf};

fn main() {
    println!("=== File Path Manipulation ===\n");
    
    // Directory and basename extraction
    println!("Directory and basename extraction:");
    let path = Path::new("/home/user/documents/file.txt");
    extract_components(path);
    println!();
    
    // Extension operations
    println!("Extension operations:");
    let files = vec!["document.txt", "program.rs", "archive.tar.gz", "README"];
    for file in &files {
        let path = Path::new(file);
        println!("  File: {}", file);
        println!("    Extension: '{}'", path.extension().and_then(|s| s.to_str()).unwrap_or(""));
    }
    println!();
    
    // Join paths
    println!("Join paths:");
    let joined = Path::new("/home/user").join("documents/file.txt");
    println!("  Result: {}", joined.display());
    println!();
    
    // Absolute vs relative
    println!("Path type detection:");
    println!("  /home/user: {}", if Path::new("/home/user").is_absolute() { "Absolute" } else { "Relative" });
    println!("  documents/file.txt: {}", if Path::new("documents/file.txt").is_absolute() { "Absolute" } else { "Relative" });
    println!();
    
    println!("Path manipulation in Rust:");
    println!("  - Path and PathBuf types");
    println!("  - parent(), file_name(), extension()");
    println!("  - join() for combining paths");
    println!("  - is_absolute(), is_relative()");
    println!("  - Cross-platform path handling");
}

fn extract_components(path: &Path) {
    println!("Path: {}", path.display());
    if let Some(parent) = path.parent() {
        println!("  Directory: {}", parent.display());
    }
    if let Some(filename) = path.file_name() {
        println!("  Basename: {}", filename.to_string_lossy());
    }
}

// Key differences from C:
// 1. Path and PathBuf instead of char*
// 2. Methods instead of libgen functions
// 3. Cross-platform path handling
// 4. UTF-8 validation built-in
// 5. Type-safe path operations
// 6. No manual memory management
