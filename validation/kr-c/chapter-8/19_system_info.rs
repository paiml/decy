/* K&R C Chapter 8: System Information
 * Transpiled to safe Rust (using sysinfo crate pattern)
 */

use std::env;

fn main() {
    println!("=== System Information ===\n");
    
    demo_basic_info();
    
    println!("System information in Rust:");
    println!("  - std::env for basic info");
    println!("  - sysinfo crate for detailed system info");
    println!("  - num_cpus crate for CPU count");
    println!("  - No direct uname/sysconf equivalents");
    
    println!("\nExample with sysinfo crate:");
    println!("```rust");
    println!("use sysinfo::{{System, SystemExt}};");
    println!();
    println!("let mut sys = System::new_all();");
    println!("sys.refresh_all();");
    println!();
    println!("println!(\"OS: {{:?}}\", sys.name());");
    println!("println!(\"Kernel: {{:?}}\", sys.kernel_version());");
    println!("println!(\"OS version: {{:?}}\", sys.os_version());");
    println!("println!(\"CPUs: {{}}\", sys.cpus().len());");
    println!("println!(\"Memory: {{}} KB\", sys.total_memory());");
    println!("```");
    
    println!("\nExample with num_cpus crate:");
    println!("```rust");
    println!("let cpus = num_cpus::get();");
    println!("println!(\"CPUs: {{}}\", cpus);");
    println!("```");
}

fn demo_basic_info() {
    println!("=== Basic System Info ===");
    
    println!("OS: {}", env::consts::OS);
    println!("Architecture: {}", env::consts::ARCH);
    println!("Family: {}", env::consts::FAMILY);
    
    if let Ok(current_dir) = env::current_dir() {
        println!("Current directory: {}", current_dir.display());
    }
    
    println!();
}

// Key differences from C:
// 1. No direct uname equivalent
// 2. sysinfo crate for system info
// 3. num_cpus crate for CPU count
// 4. env::consts for OS/arch
// 5. Cross-platform abstractions
// 6. No sysconf/getrlimit in std
