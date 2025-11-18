/* K&R C Chapter 7.5: Random Access - fseek and ftell
 * Page 163
 * Transpiled to safe Rust (using Seek trait)
 */

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};

fn main() -> io::Result<()> {
    println!("=== Random Access (fseek/ftell alternatives) ===\n");

    // Create file with numbers
    let filename = "positions.txt";
    let mut file = File::create(filename)?;

    for i in 0..10 {
        writeln!(file, "{}", i * 10)?;
    }

    println!("Created file with 10 numbers");

    // Reopen for reading with random access
    let mut file = File::open(filename)?;
    let mut buffer = String::new();

    // Read normally
    file.read_to_string(&mut buffer)?;
    let first_line = buffer.lines().next().unwrap_or("");
    println!("First number: {}", first_line);

    // Reopen to reset position (alternative: seek to start)
    drop(file);
    let mut file = File::open(filename)?;

    // Read first line properly
    let mut first = String::new();
    use std::io::BufRead;
    let mut reader = io::BufReader::new(file);
    reader.read_line(&mut first)?;
    println!("First number: {}", first.trim());

    // Save position (ftell equivalent)
    let pos = reader.stream_position()?;
    println!("Current position: {} bytes", pos);

    // Seek to end (SEEK_END)
    let end_pos = reader.seek(SeekFrom::End(0))?;
    println!("File size: {} bytes", end_pos);

    // Seek back to saved position (SEEK_SET)
    reader.seek(SeekFrom::Start(pos))?;
    let mut second = String::new();
    reader.read_line(&mut second)?;
    println!("Next number: {}", second.trim());

    // Demonstrate more seeking
    demo_seeking()?;

    Ok(())
}

fn demo_seeking() -> io::Result<()> {
    println!("\n=== Advanced Seeking ===\n");

    let filename = "seek_demo.txt";
    let mut file = File::create(filename)?;

    // Write some data
    writeln!(file, "Line 1: First line")?;
    writeln!(file, "Line 2: Second line")?;
    writeln!(file, "Line 3: Third line")?;

    drop(file);

    // Open for reading and seeking
    let mut file = File::open(filename)?;

    // Seek from start
    file.seek(SeekFrom::Start(0))?;
    println!("Position after SeekFrom::Start(0): {}", file.stream_position()?);

    // Seek from current position
    file.seek(SeekFrom::Current(10))?;
    println!("Position after SeekFrom::Current(10): {}", file.stream_position()?);

    // Seek from end
    file.seek(SeekFrom::End(-10))?;
    println!("Position after SeekFrom::End(-10): {}", file.stream_position()?);

    // Read from current position
    let mut buffer = [0u8; 10];
    let bytes_read = file.read(&mut buffer)?;
    println!("Read {} bytes: {:?}", bytes_read, std::str::from_utf8(&buffer[..bytes_read]));

    // Get file size
    let size = file.seek(SeekFrom::End(0))?;
    println!("File size: {} bytes", size);

    // Rewind (seek to start)
    file.seek(SeekFrom::Start(0))?;
    println!("After rewind: position = {}", file.stream_position()?);

    Ok(())
}

// Random access within a file
#[allow(dead_code)]
fn random_access_example() -> io::Result<()> {
    let filename = "random.dat";
    let mut file = File::create(filename)?;

    // Write numbers at specific positions
    for i in 0..10 {
        let pos = (i * 8) as u64;  // 8 bytes per i64
        file.seek(SeekFrom::Start(pos))?;
        let value = (i * 100) as i64;
        file.write_all(&value.to_le_bytes())?;
    }

    drop(file);

    // Read numbers in random order
    let mut file = File::open(filename)?;
    let indices = [5, 2, 8, 1, 9];

    for &idx in &indices {
        let pos = (idx * 8) as u64;
        file.seek(SeekFrom::Start(pos))?;

        let mut bytes = [0u8; 8];
        file.read_exact(&mut bytes)?;
        let value = i64::from_le_bytes(bytes);

        println!("Index {}: value = {}", idx, value);
    }

    Ok(())
}

// Key differences from C:
// 1. Seek trait instead of fseek/ftell
// 2. SeekFrom enum for positioning
// 3. stream_position() instead of ftell
// 4. seek() returns Result<u64>
// 5. Type-safe seeking
// 6. No SEEK_SET/SEEK_CUR/SEEK_END macros
// 7. Automatic error handling
// 8. BufReader for buffered I/O
