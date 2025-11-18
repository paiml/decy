/* K&R C Chapter 8: Daemon Process Creation
 * Transpiled to safe Rust (using daemonize crate pattern)
 */

use std::fs::OpenOptions;
use std::io::Write;
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Daemon Process Creation ===\n");
    
    println!("Daemon characteristics:");
    println!("  1. Run in background");
    println!("  2. Detached from terminal");
    println!("  3. No controlling terminal");
    println!("  4. Changed working directory");
    println!("  5. Closed inherited file descriptors");
    println!("  6. Redirected stdio");
    
    println!("\nRust daemon patterns:");
    println!("  - daemonize crate: Proper daemonization");
    println!("  - No manual fork/setsid in safe Rust");
    println!("  - Use systemd/launchd for production");
    
    println!("\nExample with daemonize crate:");
    println!("```rust");
    println!("use daemonize::Daemonize;");
    println!("use std::fs::File;");
    println!();
    println!("let stdout = File::create(\"/tmp/daemon.out\").unwrap();");
    println!("let stderr = File::create(\"/tmp/daemon.err\").unwrap();");
    println!();
    println!("let daemonize = Daemonize::new()");
    println!("    .pid_file(\"/tmp/daemon.pid\")");
    println!("    .chown_pid_file(true)");
    println!("    .working_directory(\"/tmp\")");
    println!("    .user(\"nobody\")");
    println!("    .group(\"daemon\")");
    println!("    .stdout(stdout)");
    println!("    .stderr(stderr);");
    println!();
    println!("match daemonize.start() {{");
    println!("    Ok(_) => println!(\"Success, daemonized\"),");
    println!("    Err(e) => eprintln!(\"Error: {{}}\", e),");
    println!("}}");
    println!("```");
    
    println!("\nSimple background service pattern:");
    demo_service_pattern();
}

fn demo_service_pattern() {
    println!();
    println!("=== Service Pattern Demo ===");
    
    write_log("Service started").ok();
    
    for i in 0..3 {
        thread::sleep(Duration::from_secs(1));
        write_log(&format!("Service iteration {}", i)).ok();
    }
    
    write_log("Service stopped").ok();
    
    println!("Check log: cat /tmp/service_test.log");
}

fn write_log(message: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/service_test.log")?;
    
    writeln!(file, "{}", message)?;
    
    Ok(())
}

// Key differences from C:
// 1. daemonize crate instead of manual fork
// 2. No raw fork/setsid in safe Rust
// 3. Use systemd for production daemons
// 4. Service configuration files preferred
// 5. No manual FD manipulation
// 6. Cross-platform service patterns
