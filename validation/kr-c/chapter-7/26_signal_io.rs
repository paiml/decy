/* K&R C Chapter 7: Signal Handling with I/O
 * Transpiled to safe Rust (using channels and timeouts)
 */

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() -> io::Result<()> {
    println!("=== Signal Handling with I/O (Rust Pattern) ===\n");
    
    timed_read_demo()?;
    channel_based_interruption()?;
    
    println!("Signal handling in Rust:");
    println!("  - No direct signal API (use signal-hook crate)");
    println!("  - Channels for inter-thread communication");
    println!("  - Timeout-based operations");
    println!("  - No EINTR - handled by std library");
    
    Ok(())
}

fn timed_read_demo() -> io::Result<()> {
    println!("=== Timed Read Demo ===");
    
    let (tx, rx) = mpsc::channel();
    
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        tx.send("Data arrived").ok();
    });
    
    println!("Waiting for data with 2-second timeout...");
    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok(data) => println!("  Received: {}", data),
        Err(_) => println!("  Timeout!"),
    }
    println!();
    
    Ok(())
}

fn channel_based_interruption() -> io::Result<()> {
    println!("=== Channel-Based Interruption ===");
    
    let (tx, rx) = mpsc::channel();
    
    let handle = thread::spawn(move || {
        loop {
            match rx.try_recv() {
                Ok(_) => {
                    println!("  Received interrupt signal");
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    thread::sleep(Duration::from_millis(100));
                }
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        }
    });
    
    thread::sleep(Duration::from_millis(500));
    println!("  Sending interrupt...");
    tx.send(()).ok();
    
    handle.join().ok();
    println!();
    
    Ok(())
}

// Key differences from C:
// 1. Channels instead of signals
// 2. recv_timeout for timed operations
// 3. No SIGINT/SIGALRM - use channels
// 4. Thread-safe communication
// 5. No signal-unsafe functions
