/* K&R C Chapter 8.6: Directory Listing
 * Transpiled to safe Rust (using fs::read_dir)
 */

use std::env;
use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let dirname = if args.len() > 1 { &args[1] } else { "." };
    
    println!("Contents of {}:", dirname);
    
    for entry in fs::read_dir(dirname)? {
        let entry = entry?;
        let filename = entry.file_name();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirEntryExt;
            println!("  {} (inode: {})", filename.to_string_lossy(), entry.ino());
        }
        
        #[cfg(not(unix))]
        {
            println!("  {}", filename.to_string_lossy());
        }
    }
    
    Ok(())
}

// Key differences from C:
// 1. fs::read_dir iterator
// 2. DirEntry type
// 3. No manual opendir/closedir
// 4. RAII: automatic cleanup
// 5. Iterator pattern
// 6. OsString for filenames
