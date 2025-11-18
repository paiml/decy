/* K&R C Chapter 8: Signal Handling
 * Transpiled to safe Rust (using ctrlc crate pattern)
 */

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Signal Handling (Rust Pattern) ===\n");
    
    demo_ctrlc_pattern();
    
    println!("\nSignal handling in Rust:");
    println!("  - No direct signal API in std");
    println!("  - Use ctrlc crate for SIGINT/SIGTERM");
    println!("  - Use signal-hook crate for full signals");
    println!("  - Channels for custom events");
    println!("  - atomic types for signal flags");
    
    println!("\nExample with ctrlc crate:");
    println!("```rust");
    println!("use ctrlc;");
    println!("use std::sync::atomic::{{AtomicBool, Ordering}};");
    println!("use std::sync::Arc;");
    println!();
    println!("let running = Arc::new(AtomicBool::new(true));");
    println!("let r = running.clone();");
    println!();
    println!("ctrlc::set_handler(move || {{");
    println!("    r.store(false, Ordering::SeqCst);");
    println!("}}).expect(\"Error setting Ctrl-C handler\");");
    println!();
    println!("while running.load(Ordering::SeqCst) {{");
    println!("    // Work");
    println!("}}");
    println!("```");
}

fn demo_ctrlc_pattern() {
    println!("=== Pattern: Atomic Flag for Interruption ===");
    
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    // Simulate timeout instead of actual signal
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));
        println!("  (Simulating SIGALRM)");
        r.store(false, Ordering::SeqCst);
    });
    
    println!("Running for 2 seconds (or until interrupted)...");
    
    let mut count = 0;
    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
        count += 1;
    }
    
    println!("Stopped after {} iterations", count);
}

// Key differences from C:
// 1. No signal() function in std
// 2. Use ctrlc crate for SIGINT/SIGTERM
// 3. Use signal-hook for other signals
// 4. Atomic types for signal flags
// 5. Thread-safe signal handling
// 6. No signal-unsafe functions
