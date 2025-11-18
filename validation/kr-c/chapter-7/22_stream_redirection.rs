/* K&R C Chapter 7: Stream Redirection
 * Transpiled to safe Rust (using std::process::Command)
 */

use std::fs::File;
use std::io::{self, Write};
use std::process::{Command, Stdio};

fn main() -> io::Result<()> {
    println!("=== Stream Redirection ===\n");
    
    test_stdout_redirection()?;
    test_command_redirection()?;
    
    println!("Stream redirection in Rust:");
    println!("  - Command::stdout for process redirection");
    println!("  - File::create for output redirection");
    println!("  - Stdio::piped() for capturing output");
    println!("  - No low-level dup/dup2 needed");
    
    Ok(())
}

fn test_stdout_redirection() -> io::Result<()> {
    println!("=== File Output Redirection ===");
    
    let mut file = File::create("stdout_redirect.txt")?;
    writeln!(file, "Line 1: This is redirected to file")?;
    writeln!(file, "Line 2: More redirected output")?;
    drop(file);
    
    let contents = std::fs::read_to_string("stdout_redirect.txt")?;
    println!("File contents:\n{}", contents);
    
    std::fs::remove_file("stdout_redirect.txt")?;
    Ok(())
}

fn test_command_redirection() -> io::Result<()> {
    println!("=== Command Output Redirection ===");
    
    let output = Command::new("echo")
        .arg("Hello from command")
        .output()?;
    
    println!("Command output: {}", String::from_utf8_lossy(&output.stdout));
    
    // Redirect to file
    let file = File::create("command_output.txt")?;
    Command::new("echo")
        .arg("Redirected to file")
        .stdout(Stdio::from(file))
        .spawn()?
        .wait()?;
    
    let contents = std::fs::read_to_string("command_output.txt")?;
    println!("File contents: {}", contents);
    
    std::fs::remove_file("command_output.txt")?;
    Ok(())
}

// Key differences from C:
// 1. Command API instead of dup/dup2
// 2. Stdio enum for redirection
// 3. output() method captures stdout/stderr
// 4. Type-safe process spawning
// 5. No file descriptor manipulation
