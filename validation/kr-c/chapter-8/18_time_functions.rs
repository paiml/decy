/* K&R C Chapter 8: Time and Date Functions
 * Transpiled to safe Rust (using std::time and chrono crate)
 */

use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn main() {
    println!("=== Time and Date Functions ===\n");
    
    demo_system_time();
    demo_instant();
    demo_duration();
    
    println!("Time handling in Rust:");
    println!("  - SystemTime: Wall clock time");
    println!("  - Instant: Monotonic time (for durations)");
    println!("  - Duration: Time spans");
    println!("  - chrono crate: Full date/time formatting");
    
    println!("\nExample with chrono crate:");
    println!("```rust");
    println!("use chrono::{{Local, Utc}};");
    println!();
    println!("let now = Local::now();");
    println!("println!(\"{{:?}}\", now);");
    println!("println!(\"{{:?}}\", now.format(\"%Y-%m-%d %H:%M:%S\"));");
    println!("```");
}

fn demo_system_time() {
    println!("=== SystemTime Demo ===");
    
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    
    println!("Unix timestamp: {} seconds since epoch", 
             since_epoch.as_secs());
    println!();
}

fn demo_instant() {
    println!("=== Instant Demo (Monotonic Time) ===");
    
    let start = Instant::now();
    
    // Simulate work
    thread::sleep(Duration::from_millis(100));
    
    let elapsed = start.elapsed();
    println!("Elapsed: {:.3} seconds", elapsed.as_secs_f64());
    println!();
}

fn demo_duration() {
    println!("=== Duration Demo ===");
    
    let d1 = Duration::from_secs(60);
    let d2 = Duration::from_millis(500);
    let total = d1 + d2;
    
    println!("Duration: {} seconds", total.as_secs());
    println!("Duration: {} milliseconds", total.as_millis());
    println!("Duration: {} microseconds", total.as_micros());
    println!();
}

// Key differences from C:
// 1. SystemTime for wall clock
// 2. Instant for monotonic time
// 3. Duration type for spans
// 4. chrono crate for formatting
// 5. Type-safe time operations
// 6. No struct tm - use chrono
