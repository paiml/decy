/* K&R C Chapter 7: Directory Operations
 * Transpiled to safe Rust (using std::fs::read_dir)
 */

use std::fs;
use std::path::Path;

fn main() -> std::io::Result<()> {
    println!("=== Directory Operations ===\n");
    
    fs::create_dir_all("test_dir/subdir1")?;
    fs::write("test_dir/file1.txt", "content")?;
    fs::write("test_dir/file2.rs", "fn main() {}")?;
    
    println!("Basic directory listing:");
    list_directory("test_dir")?;
    println!();
    
    println!("Filter by extension:");
    list_by_extension("test_dir", "txt")?;
    println!();
    
    println!("Sorted listing:");
    list_sorted("test_dir")?;
    println!();
    
    fs::remove_dir_all("test_dir")?;
    
    Ok(())
}

fn list_directory(path: &str) -> std::io::Result<()> {
    println!("Contents of directory: {}", path);
    
    for (i, entry) in fs::read_dir(path)?.enumerate() {
        let entry = entry?;
        let name = entry.file_name();
        let file_type = if entry.path().is_dir() { " (directory)" } else { " (file)" };
        println!("  [{}] {}{}", i + 1, name.to_string_lossy(), file_type);
    }
    
    Ok(())
}

fn list_by_extension(path: &str, ext: &str) -> std::io::Result<()> {
    println!("Files with extension '.{}' in {}:", ext, path);
    
    let mut count = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some(ext) {
            count += 1;
            println!("  [{}] {}", count, entry.file_name().to_string_lossy());
        }
    }
    
    println!("Found {} files", count);
    Ok(())
}

fn list_sorted(path: &str) -> std::io::Result<()> {
    println!("Directory {} (sorted):", path);
    
    let mut entries: Vec<_> = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    
    entries.sort();
    
    for (i, name) in entries.iter().enumerate() {
        println!("  [{}] {}", i + 1, name);
    }
    
    Ok(())
}

// Key differences from C:
// 1. fs::read_dir instead of opendir/readdir
// 2. Iterator over DirEntry
// 3. Path methods for file operations
// 4. No manual closedir needed (RAII)
// 5. Cross-platform path handling
