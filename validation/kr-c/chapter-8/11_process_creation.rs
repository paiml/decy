/* K&R C Chapter 8: Process Creation and Management
 * Transpiled to safe Rust (using std::process::Command)
 */

use std::process::{Command, Stdio};

fn main() {
    println!("=== Process Creation ===\n");
    
    demo_command();
    demo_command_output();
    demo_exit_status();
    
    println!("Process management in Rust:");
    println!("  - Command: Process builder");
    println!("  - spawn(): Non-blocking execution");
    println!("  - output(): Blocking, captures output");
    println!("  - status(): Just wait for exit");
    println!("  - No fork() - use Command");
}

fn demo_command() {
    println!("=== Command Demo ===");
    
    let status = Command::new("echo")
        .arg("Hello from child process")
        .status()
        .expect("Failed to execute command");
    
    println!("Command exited with: {}", status);
    println!();
}

fn demo_command_output() {
    println!("=== Command Output Demo ===");
    
    let output = Command::new("ls")
        .arg("-l")
        .arg("/tmp")
        .output()
        .expect("Failed to execute command");
    
    println!("Exit status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!();
}

fn demo_exit_status() {
    println!("=== Exit Status Demo ===");
    
    // Command that exits with code 42
    let status = Command::new("sh")
        .arg("-c")
        .arg("exit 42")
        .status()
        .expect("Failed to execute command");
    
    if let Some(code) = status.code() {
        println!("Child exited with status: {}", code);
    }
    
    println!();
}

// Key differences from C:
// 1. Command builder instead of fork/exec
// 2. No manual process management
// 3. Type-safe process API
// 4. RAII: automatic cleanup
// 5. Cross-platform abstraction
// 6. No wait() - built into Command
