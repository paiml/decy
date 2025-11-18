/* K&R C Chapter 7: Pipeline and Pipe I/O
 * Transpiled to safe Rust (using std::process::Command)
 */

use std::io::{Read, Write};
use std::process::{Command, Stdio};

fn main() -> std::io::Result<()> {
    println!("=== Pipeline and Pipe I/O ===\n");
    
    simple_pipe_demo()?;
    pipeline_demo()?;
    
    println!("Pipe I/O in Rust:");
    println!("  - Command::stdout(Stdio::piped())");
    println!("  - Command::stdin(Stdio::from(child.stdout))");
    println!("  - Higher-level than C pipes");
    println!("  - Automatic resource cleanup (RAII)");
    
    Ok(())
}

fn simple_pipe_demo() -> std::io::Result<()> {
    println!("=== Simple Pipe Demo ===");
    
    let output = Command::new("echo")
        .arg("Hello from command")
        .output()?;
    
    println!("Output: {}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

fn pipeline_demo() -> std::io::Result<()> {
    println!("=== Pipeline Demo ===");
    
    // Pipeline: echo | tr (transform)
    let echo = Command::new("echo")
        .arg("hello world")
        .stdout(Stdio::piped())
        .spawn()?;
    
    let tr = Command::new("tr")
        .arg("a-z")
        .arg("A-Z")
        .stdin(Stdio::from(echo.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()?;
    
    let output = tr.wait_with_output()?;
    println!("Pipeline output: {}", String::from_utf8_lossy(&output.stdout));
    
    Ok(())
}

// Key differences from C:
// 1. Command API instead of pipe()/fork()
// 2. Stdio enum for stream configuration
// 3. RAII: automatic cleanup
// 4. Type-safe process spawning
// 5. No manual file descriptor manipulation
// 6. Cross-platform API
