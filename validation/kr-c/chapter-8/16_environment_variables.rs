/* K&R C Chapter 8: Environment Variables
 * Transpiled to safe Rust (using std::env)
 */

use std::env;

fn main() {
    println!("=== Environment Variables ===\n");
    
    demo_env_var();
    demo_set_var();
    demo_vars();
    
    println!("Environment functions:");
    println!("  - env::var(): Get variable (Result)");
    println!("  - env::set_var(): Set variable");
    println!("  - env::remove_var(): Remove variable");
    println!("  - env::vars(): Iterator over all variables");
}

fn demo_env_var() {
    println!("=== env::var() Demo ===");
    
    if let Ok(home) = env::var("HOME") {
        println!("HOME: {}", home);
    }
    
    if let Ok(path) = env::var("PATH") {
        println!("PATH: {}", path);
    }
    
    if let Ok(user) = env::var("USER") {
        println!("USER: {}", user);
    }
    
    println!();
}

fn demo_set_var() {
    println!("=== env::set_var() Demo ===");
    
    env::set_var("MY_VAR", "HelloWorld");
    
    if let Ok(value) = env::var("MY_VAR") {
        println!("MY_VAR: {}", value);
    }
    
    env::set_var("MY_VAR", "NewValue");
    if let Ok(value) = env::var("MY_VAR") {
        println!("MY_VAR (updated): {}", value);
    }
    
    env::remove_var("MY_VAR");
    match env::var("MY_VAR") {
        Ok(value) => println!("MY_VAR (after remove): {}", value),
        Err(_) => println!("MY_VAR (after remove): (not set)"),
    }
    
    println!();
}

fn demo_vars() {
    println!("=== env::vars() Demo ===");
    println!("All environment variables:");
    
    let vars: Vec<_> = env::vars().collect();
    
    for (key, value) in vars.iter().take(5) {
        println!("  {}={}", key, value);
    }
    
    println!("  ... ({} total variables)", vars.len());
    println!();
}

// Key differences from C:
// 1. env::var() returns Result<String, _>
// 2. env::set_var() for modification
// 3. env::vars() iterator
// 4. No global environ array
// 5. Type-safe String values
// 6. No null pointer handling
