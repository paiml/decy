/* K&R C Chapter 8: File Descriptor Duplication
 * Transpiled to safe Rust (using std::process for redirection)
 */

use std::fs::File;
use std::io::{self, Write};
use std::process::{Command, Stdio};

fn main() -> io::Result<()> {
    println!("=== File Descriptor Duplication (Rust Pattern) ===\n");
    
    println!("Note: Rust doesn't expose dup/dup2 directly");
    println!("Instead, use high-level abstractions:\n");
    
    // Demonstrate file writing with multiple handles
    demo_shared_file()?;
    
    // Demonstrate stdout redirection
    demo_stdout_redirect()?;
    
    println!("Rust patterns for FD manipulation:");
    println!("  - File::try_clone() for duplicating handles");
    println!("  - Command with Stdio for redirection");
    println!("  - No raw file descriptor manipulation");
    println!("  - RAII ensures proper cleanup");
    
    Ok(())
}

fn demo_shared_file() -> io::Result<()> {
    println!("=== Shared File Demo ===");
    
    let mut file1 = File::create("shared_test.txt")?;
    
    // Clone the file handle (similar to dup)
    let mut file2 = file1.try_clone()?;
    
    file1.write_all(b"From file1\n")?;
    file2.write_all(b"From file2\n")?;
    
    drop(file1);
    file2.write_all(b"Still works after file1 closed\n")?;
    
    drop(file2);
    
    let contents = std::fs::read_to_string("shared_test.txt")?;
    println!("File contents:\n{}", contents);
    
    std::fs::remove_file("shared_test.txt")?;
    
    Ok(())
}

fn demo_stdout_redirect() -> io::Result<()> {
    println!("=== Stdout Redirection Demo ===");
    
    let output_file = File::create("redirect_test.txt")?;
    
    let output = Command::new("echo")
        .arg("This goes to redirect_test.txt")
        .stdout(Stdio::from(output_file))
        .output()?;
    
    println!("Command exit status: {:?}", output.status);
    
    let contents = std::fs::read_to_string("redirect_test.txt")?;
    println!("Redirected output: {}", contents.trim());
    
    std::fs::remove_file("redirect_test.txt")?;
    
    Ok(())
}

// Key differences from C:
// 1. try_clone() instead of dup()
// 2. Command with Stdio for redirection
// 3. No raw file descriptor manipulation
// 4. Type-safe handle operations
// 5. RAII: automatic close
// 6. No dup2 equivalent - use Command API
