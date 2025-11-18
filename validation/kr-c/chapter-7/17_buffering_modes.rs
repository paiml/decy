/* K&R C Chapter 7: Buffering Modes
 * Transpiled to safe Rust (BufWriter, flush)
 */

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::thread::sleep;
use std::time::Duration;

fn main() -> io::Result<()> {
    println!("=== Buffering Modes ===\n");
    
    demo_unbuffered()?;
    demo_buffered()?;
    demo_manual_flush()?;

    println!("Buffering in Rust:");
    println!("  - BufWriter for buffered output");
    println!("  - flush() to force immediate output");
    println!("  - Automatic flush on drop");
    
    Ok(())
}

fn demo_unbuffered() -> io::Result<()> {
    println!("=== Unbuffered I/O (direct writes) ===");
    
    let mut file = File::create("unbuffered.txt")?;
    
    println!("Writing with unbuffered mode...");
    writeln!(file, "Line 1")?;
    file.flush()?;  // Force immediate write
    println!("  (written immediately)");
    sleep(Duration::from_secs(1));
    
    writeln!(file, "Line 2")?;
    file.flush()?;
    println!("  (written immediately)\n");
    
    Ok(())
}

fn demo_buffered() -> io::Result<()> {
    println!("=== Buffered I/O (BufWriter) ===");
    
    let file = File::create("buffered.txt")?;
    let mut writer = BufWriter::new(file);
    
    println!("Writing with buffered mode...");
    writeln!(writer, "Line 1")?;
    writeln!(writer, "Line 2")?;
    println!("  (buffered in memory)\n");
    sleep(Duration::from_secs(1));
    
    println!("Flushing buffer...");
    writer.flush()?;
    println!("  (now written to file)\n");
    
    Ok(())
}

fn demo_manual_flush() -> io::Result<()> {
    println!("=== Manual Flushing ===");
    
    print!("Progress: ");
    io::stdout().flush()?;
    
    for _ in 0..10 {
        print!("#");
        io::stdout().flush()?;  // Force immediate output
        sleep(Duration::from_millis(200));
    }
    println!(" Done!\n");
    
    Ok(())
}

// Key differences from C:
// 1. BufWriter instead of setvbuf
// 2. flush() method instead of fflush
// 3. Automatic flush on drop (RAII)
// 4. No _IONBF/_IOLBF/_IOFBF modes
// 5. Type-safe buffering
// 6. Buffer size tunable via with_capacity
