/* K&R C Chapter 8: Recursive Directory Traversal
 * Transpiled to safe Rust (using walkdir crate pattern)
 */

use std::fs;
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    println!("=== Recursive Directory Traversal ===\n");
    
    let path = "/tmp";
    println!("Traversing: {}", path);
    
    traverse_directory(Path::new(path), 0, 3)?;
    
    println!("\nDirectory traversal in Rust:");
    println!("  - Manual: fs::read_dir + recursion");
    println!("  - walkdir crate: WalkDir iterator");
    println!("  - ignore crate: Gitignore-aware traversal");
    
    println!("\nExample with walkdir crate:");
    println!("```rust");
    println!("use walkdir::WalkDir;");
    println!();
    println!("for entry in WalkDir::new(\"/tmp\").max_depth(3) {{");
    println!("    let entry = entry?;");
    println!("    println!(\"{{:?}}\", entry.path());");
    println!("}}");
    println!("```");
    
    Ok(())
}

fn traverse_directory(path: &Path, depth: usize, max_depth: usize) -> io::Result<()> {
    if depth > max_depth {
        return Ok(());
    }
    
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let filename = entry.file_name();
        
        print_indent(depth);
        
        if path.is_dir() {
            println!("[DIR]  {}/", filename.to_string_lossy());
            traverse_directory(&path, depth + 1, max_depth)?;
        } else {
            let metadata = entry.metadata()?;
            println!("[FILE] {} ({} bytes)", 
                     filename.to_string_lossy(), 
                     metadata.len());
        }
    }
    
    Ok(())
}

fn print_indent(depth: usize) {
    for _ in 0..depth {
        print!("  ");
    }
}

// Key differences from C:
// 1. fs::read_dir iterator
// 2. Path type for paths
// 3. Recursive with Result propagation
// 4. walkdir crate for production
// 5. No manual buffer sizing
// 6. Type-safe path operations
