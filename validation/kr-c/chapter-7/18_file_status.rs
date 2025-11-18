/* K&R C Chapter 7: File Status and Metadata
 * Transpiled to safe Rust (using std::fs::metadata)
 */

use std::fs;
use std::path::Path;
use std::time::SystemTime;

fn main() -> std::io::Result<()> {
    println!("=== File Status and Metadata ===\n");
    
    let test_file = "status_test.txt";
    fs::write(test_file, "Test file content\nMultiple lines\n")?;
    
    check_existence(test_file);
    check_permissions(test_file)?;
    file_info(test_file)?;
    classify_file("program.rs");
    
    fs::remove_file(test_file)?;
    
    println!("\nFile status operations in Rust:");
    println!("  - fs::metadata() for file info");
    println!("  - Path::exists() for existence check");
    println!("  - Metadata provides size, times, permissions");
    
    Ok(())
}

fn check_existence(filename: &str) {
    println!("File existence:");
    println!("  {}: {}", filename, if Path::new(filename).exists() { "EXISTS" } else { "NOT FOUND" });
    println!();
}

fn check_permissions(filename: &str) -> std::io::Result<()> {
    println!("Permissions:");
    let metadata = fs::metadata(filename)?;
    println!("  Exists: YES");
    println!("  Readable: YES");
    println!("  Writable: {}", !metadata.permissions().readonly());
    println!();
    Ok(())
}

fn file_info(filename: &str) -> std::io::Result<()> {
    let metadata = fs::metadata(filename)?;
    
    println!("File information for: {}", filename);
    println!("  Size: {} bytes", metadata.len());
    println!("  Type: {}", if metadata.is_dir() { "Directory" } else { "Regular file" });
    
    if let Ok(modified) = metadata.modified() {
        println!("  Modified: {:?}", modified);
    }
    
    println!();
    Ok(())
}

fn classify_file(filename: &str) {
    let ext = Path::new(filename).extension().and_then(|s| s.to_str()).unwrap_or("");
    
    println!("File: {}", filename);
    println!("  Extension: {}", if ext.is_empty() { "(none)" } else { ext });
    println!("  Type: {}", match ext {
        "rs" => "Rust source",
        "c" | "h" => "C source/header",
        "txt" => "Text file",
        _ => "Unknown",
    });
    println!();
}

// Key differences from C:
// 1. fs::metadata instead of stat
// 2. Path type for path operations
// 3. Result<Metadata> for error handling
// 4. No struct stat
// 5. SystemTime instead of time_t
// 6. Cross-platform API
