/* K&R C Chapter 7: I/O Performance Patterns
 * Transpiled to safe Rust (using BufWriter/BufReader)
 */

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::time::Instant;

fn main() -> io::Result<()> {
    println!("=== I/O Performance Patterns ===\n");
    
    let test_file = "perf_test.txt";
    
    benchmark_unbuffered(test_file)?;
    benchmark_buffered(test_file)?;
    
    std::fs::remove_file(test_file)?;
    
    println!("Performance tips:");
    println!("  1. Use BufWriter for buffered output");
    println!("  2. Use BufReader for buffered input");
    println!("  3. Batch operations where possible");
    println!("  4. Consider io::copy for large transfers");
    
    Ok(())
}

fn benchmark_unbuffered(filename: &str) -> io::Result<()> {
    println!("=== Unbuffered Write ===");
    
    let start = Instant::now();
    let mut file = File::create(filename)?;
    
    for i in 0..10000 {
        write!(file, "Line {}\n", i)?;
    }
    
    let duration = start.elapsed();
    println!("  Time: {:.6} seconds\n", duration.as_secs_f64());
    
    Ok(())
}

fn benchmark_buffered(filename: &str) -> io::Result<()> {
    println!("=== Buffered Write ===");
    
    let start = Instant::now();
    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);
    
    for i in 0..10000 {
        writeln!(writer, "Line {}", i)?;
    }
    
    writer.flush()?;
    
    let duration = start.elapsed();
    println!("  Time: {:.6} seconds\n", duration.as_secs_f64());
    
    Ok(())
}

// Key differences from C:
// 1. BufWriter/BufReader wrappers
// 2. Instant for timing (monotonic clock)
// 3. No manual buffer allocation
// 4. RAII: automatic flush on drop
// 5. Type-safe buffering
