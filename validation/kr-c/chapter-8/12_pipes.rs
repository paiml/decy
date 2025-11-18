/* K&R C Chapter 8: Inter-Process Communication with Pipes
 * Transpiled to safe Rust (using Command with Stdio)
 */

use std::io::{self, Write};
use std::process::{Command, Stdio};

fn main() -> io::Result<()> {
    println!("=== Pipes (IPC) ===\n");
    
    demo_simple_pipe()?;
    demo_pipe_chain()?;
    
    println!("Pipes in Rust:");
    println!("  - Command with Stdio::piped()");
    println!("  - ChildStdout/ChildStdin");
    println!("  - No manual pipe() call");
    println!("  - RAII: automatic cleanup");
    
    Ok(())
}

fn demo_simple_pipe() -> io::Result<()> {
    println!("=== Simple Pipe Demo ===");
    
    let mut child = Command::new("echo")
        .arg("Hello from parent!")
        .stdout(Stdio::piped())
        .spawn()?;
    
    let output = child.wait_with_output()?;
    
    println!("Child output: {}", String::from_utf8_lossy(&output.stdout));
    println!();
    
    Ok(())
}

fn demo_pipe_chain() -> io::Result<()> {
    println!("=== Pipe Chain Demo (echo | wc -l) ===");
    
    // Create echo process
    let echo = Command::new("echo")
        .arg("Line 1\nLine 2\nLine 3")
        .stdout(Stdio::piped())
        .spawn()?;
    
    // Create wc process, taking input from echo
    let wc = Command::new("wc")
        .arg("-l")
        .stdin(Stdio::from(echo.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()?;
    
    let output = wc.wait_with_output()?;
    
    println!("Line count: {}", String::from_utf8_lossy(&output.stdout).trim());
    println!();
    
    Ok(())
}

// Key differences from C:
// 1. Command with Stdio for pipes
// 2. No manual pipe() syscall
// 3. Type-safe pipe handling
// 4. RAII: automatic cleanup
// 5. No manual fork/dup2
// 6. Cross-platform API
