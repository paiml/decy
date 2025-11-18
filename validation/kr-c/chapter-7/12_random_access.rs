/* K&R C Chapter 7: Random Access File I/O
 * K&R §7.5: fseek, ftell, rewind
 * Transpiled to safe Rust (using Seek trait for random access)
 */

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};

#[derive(Debug, Clone)]
struct Record {
    id: i32,
    name: String,
    age: i32,
    score: f32,
}

fn main() -> io::Result<()> {
    println!("=== Random Access File I/O ===\n");

    let filename = "records.dat";

    // Create file with records
    let mut file = File::create(filename)?;

    // Write records (sorted by ID for binary search)
    let records = vec![
        Record {
            id: 101,
            name: "Alice".to_string(),
            age: 25,
            score: 88.5,
        },
        Record {
            id: 103,
            name: "Bob".to_string(),
            age: 30,
            score: 92.0,
        },
        Record {
            id: 105,
            name: "Carol".to_string(),
            age: 22,
            score: 85.5,
        },
        Record {
            id: 107,
            name: "David".to_string(),
            age: 28,
            score: 90.0,
        },
        Record {
            id: 109,
            name: "Eve".to_string(),
            age: 26,
            score: 94.5,
        },
    ];

    println!("Writing {} records...", records.len());
    for (i, rec) in records.iter().enumerate() {
        write_record_at(&mut file, i, rec)?;
    }

    drop(file);

    // Open for reading
    let mut file = File::open(filename)?;

    // Check file size
    let record_count = get_record_count(&mut file)?;
    println!("File contains {} records\n", record_count);

    // Random access read
    println!("Random access reads:");
    let indices = [2, 0, 4, 1];
    for &idx in &indices {
        let rec = read_record_at(&mut file, idx)?;
        println!("  Record {}: {} (ID: {})", idx, rec.name, rec.id);
    }
    println!();

    // Reverse iteration
    print_records_reverse(&mut file)?;
    println!();

    // Binary search
    println!("Binary search for ID 105:");
    match binary_search_by_id(&mut file, 105)? {
        Some((index, found)) => {
            println!(
                "  Found at index {}: {} (Age: {}, Score: {:.1})",
                index, found.name, found.age, found.score
            );
        }
        None => println!("  Not found"),
    }

    println!("\nBinary search for ID 999 (should not exist):");
    match binary_search_by_id(&mut file, 999)? {
        Some((index, _)) => println!("  Found at index {}", index),
        None => println!("  Not found (as expected)"),
    }

    // Update record in place
    println!("\nUpdating record at index 2...");
    let update = Record {
        id: 105,
        name: "Carol Smith".to_string(),
        age: 23,
        score: 87.0,
    };
    drop(file);
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(filename)?;
    write_record_at(&mut file, 2, &update)?;

    // Verify update
    let verify = read_record_at(&mut file, 2)?;
    println!(
        "  Updated record: {} (Age: {}, Score: {:.1})",
        verify.name, verify.age, verify.score
    );

    // Test stream_position and rewind
    println!("\nPosition tests:");
    println!("  Current position: {}", file.stream_position()?);
    file.seek(SeekFrom::Start(0))?;
    println!("  After rewind: {}", file.stream_position()?);
    let end_pos = file.seek(SeekFrom::End(0))?;
    println!("  At end: {} bytes", end_pos);

    println!("\nRandom access benefits:");
    println!("  - Direct record access without scanning");
    println!("  - Binary search on sorted files");
    println!("  - In-place updates");
    println!("  - Reverse iteration");

    Ok(())
}

// Calculate record size (fixed for this format)
fn record_size() -> usize {
    4 + 4 + 50 + 4 + 4  // id + name_len + name + age + score
}

// Write record at specific position
fn write_record_at<W: Write + Seek>(writer: &mut W, position: usize, rec: &Record) -> io::Result<()> {
    let offset = (position * record_size()) as u64;
    writer.seek(SeekFrom::Start(offset))?;

    writer.write_all(&rec.id.to_le_bytes())?;

    // Fixed-size name field (padded to 50 bytes)
    let mut name_bytes = [0u8; 50];
    let name = rec.name.as_bytes();
    let len = name.len().min(50);
    name_bytes[..len].copy_from_slice(&name[..len]);

    writer.write_all(&(len as u32).to_le_bytes())?;
    writer.write_all(&name_bytes)?;

    writer.write_all(&rec.age.to_le_bytes())?;
    writer.write_all(&rec.score.to_le_bytes())?;

    Ok(())
}

// Read record at specific position
fn read_record_at<R: Read + Seek>(reader: &mut R, position: usize) -> io::Result<Record> {
    let offset = (position * record_size()) as u64;
    reader.seek(SeekFrom::Start(offset))?;

    let mut id_bytes = [0u8; 4];
    reader.read_exact(&mut id_bytes)?;
    let id = i32::from_le_bytes(id_bytes);

    let mut name_len_bytes = [0u8; 4];
    reader.read_exact(&mut name_len_bytes)?;
    let name_len = u32::from_le_bytes(name_len_bytes) as usize;

    let mut name_bytes = [0u8; 50];
    reader.read_exact(&mut name_bytes)?;
    let name = String::from_utf8_lossy(&name_bytes[..name_len]).to_string();

    let mut age_bytes = [0u8; 4];
    reader.read_exact(&mut age_bytes)?;
    let age = i32::from_le_bytes(age_bytes);

    let mut score_bytes = [0u8; 4];
    reader.read_exact(&mut score_bytes)?;
    let score = f32::from_le_bytes(score_bytes);

    Ok(Record {
        id,
        name,
        age,
        score,
    })
}

// Get number of records in file
fn get_record_count<R: Read + Seek>(reader: &mut R) -> io::Result<usize> {
    let current = reader.stream_position()?;
    let filesize = reader.seek(SeekFrom::End(0))?;
    reader.seek(SeekFrom::Start(current))?;
    Ok((filesize as usize) / record_size())
}

// Print records in reverse order
fn print_records_reverse<R: Read + Seek>(reader: &mut R) -> io::Result<()> {
    let count = get_record_count(reader)?;

    println!("Records in reverse order:");
    for i in (0..count).rev() {
        let rec = read_record_at(reader, i)?;
        println!(
            "  [{}] {} (ID: {}, Age: {}, Score: {:.1})",
            i, rec.name, rec.id, rec.age, rec.score
        );
    }

    Ok(())
}

// Binary search in sorted file
fn binary_search_by_id<R: Read + Seek>(
    reader: &mut R,
    target_id: i32,
) -> io::Result<Option<(usize, Record)>> {
    let count = get_record_count(reader)?;
    let mut left = 0;
    let mut right = count as i32 - 1;

    while left <= right {
        let mid = (left + (right - left) / 2) as usize;
        let rec = read_record_at(reader, mid)?;

        if rec.id == target_id {
            return Ok(Some((mid, rec)));
        }
        if rec.id < target_id {
            left = mid as i32 + 1;
        } else {
            right = mid as i32 - 1;
        }
    }

    Ok(None)
}

// Key differences from C:
// 1. Seek trait instead of fseek/ftell
// 2. SeekFrom enum for positioning
// 3. stream_position() instead of ftell
// 4. Result<T, E> for error handling
// 5. Fixed-size records for random access
// 6. Type-safe seeking
// 7. RAII: files auto-close
// 8. No manual position tracking
