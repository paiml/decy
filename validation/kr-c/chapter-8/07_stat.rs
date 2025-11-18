/* K&R C Chapter 8.6: File Information with stat
 * Transpiled to safe Rust (using fs::metadata)
 */

use std::env;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;

fn print_file_info(filename: &str) -> io::Result<()> {
    let metadata = fs::metadata(filename)?;
    
    println!("File: {}", filename);
    println!("Size: {} bytes", metadata.len());
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        println!("Inode: {}", metadata.ino());
        println!("Mode: {:o}", metadata.permissions().mode() & 0o777);
        println!("Links: {}", metadata.nlink());
    }
    
    println!("Type: {}", if metadata.is_file() {
        "Regular file"
    } else if metadata.is_dir() {
        "Directory"
    } else if metadata.is_symlink() {
        "Symbolic link"
    } else {
        "Other"
    });
    
    if let Ok(modified) = metadata.modified() {
        println!("Last modified: {:?}", modified);
    }
    
    println!();
    
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <file1> [file2 ...]", args[0]);
        std::process::exit(1);
    }
    
    for filename in &args[1..] {
        if let Err(e) = print_file_info(filename) {
            eprintln!("Error for {}: {}", filename, e);
        }
    }
}

// Key differences from C:
// 1. fs::metadata instead of stat()
// 2. Metadata type with methods
// 3. Cross-platform API
// 4. Unix-specific traits for inode, etc.
// 5. SystemTime for timestamps
// 6. Type-safe file types
