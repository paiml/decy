/* K&R C Chapter 7: Temporary Files
 * K&R §7.5: tmpfile, tmpnam, mkstemp
 * Transpiled to safe Rust (using tempfile crate and std::env::temp_dir)
 */

use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::PathBuf;

fn main() -> io::Result<()> {
    println!("=== Temporary Files ===\n");

    demo_temp_file()?;
    demo_named_temp_file()?;
    demo_sorting_with_temp()?;
    demo_merge_sort_temp()?;
    demo_text_processing_temp()?;

    println!("\nTemporary file benefits:");
    println!("  - Automatic cleanup (when dropped)");
    println!("  - No name conflicts");
    println!("  - Useful for intermediate results");
    println!("  - External memory sorting");

    Ok(())
}

// Anonymous temporary file (like tmpfile())
fn demo_temp_file() -> io::Result<()> {
    println!("=== Anonymous temp file ===");

    // In Rust, use std::env::temp_dir() for temp directory
    // For production, use `tempfile` crate: tempfile::tempfile()
    let temp_path = std::env::temp_dir().join("rust_temp_demo.txt");
    let mut tmp = File::create(&temp_path)?;

    // Write data to temp file
    writeln!(tmp, "Line 1: Temporary data")?;
    writeln!(tmp, "Line 2: More data")?;
    writeln!(tmp, "Line 3: Final line")?;

    // Read back
    drop(tmp);
    let tmp = File::open(&temp_path)?;
    let reader = BufReader::new(tmp);

    println!("Reading from temp file:");
    for line in reader.lines() {
        println!("  {}", line?);
    }

    // Manual cleanup
    fs::remove_file(&temp_path)?;
    println!("  (temp file deleted)\n");

    Ok(())
}

// Named temporary file with auto-cleanup
fn demo_named_temp_file() -> io::Result<()> {
    println!("=== Named temporary file ===");

    // Generate unique temp filename
    let temp_path = generate_temp_filename("rust_demo");
    println!("Generated temp filename: {}", temp_path.display());

    // Create and use the file
    let mut file = File::create(&temp_path)?;
    writeln!(file, "Data in named temporary file")?;

    drop(file);

    // Read back
    let file = File::open(&temp_path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    reader.read_line(&mut buffer)?;
    print!("Contents: {}", buffer);

    // Manual cleanup
    println!("Removing temp file: {}", temp_path.display());
    fs::remove_file(&temp_path)?;
    println!();

    Ok(())
}

// Generate unique temp filename
fn generate_temp_filename(prefix: &str) -> PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let pid = std::process::id();
    let filename = format!("{}_{}_{}txt", prefix, pid, timestamp);

    std::env::temp_dir().join(filename)
}

// Temporary file for sorting large data
fn demo_sorting_with_temp() -> io::Result<()> {
    println!("=== Sorting with temporary file ===");

    let numbers = vec![64, 34, 25, 12, 22, 11, 90, 88, 45, 50, 33, 78, 21];

    // Write unsorted data to temp file
    let temp_path = generate_temp_filename("sort");
    let mut tmp = File::create(&temp_path)?;

    print!("Original numbers: ");
    for &num in &numbers {
        writeln!(tmp, "{}", num)?;
        print!("{} ", num);
    }
    println!();

    drop(tmp);

    // Read back and sort in memory
    let tmp = File::open(&temp_path)?;
    let reader = BufReader::new(tmp);

    let mut sorted: Vec<i32> = reader
        .lines()
        .filter_map(|line| line.ok()?.parse().ok())
        .collect();

    sorted.sort();

    print!("Sorted numbers:   ");
    for num in &sorted {
        print!("{} ", num);
    }
    println!();

    fs::remove_file(&temp_path)?;
    println!();

    Ok(())
}

// Multiple temp files for merge sort
fn demo_merge_sort_temp() -> io::Result<()> {
    println!("=== Merge sort with temp files ===");

    let data = vec![9, 7, 5, 3, 1, 8, 6, 4, 2, 0];

    print!("Original: ");
    for &num in &data {
        print!("{} ", num);
    }
    println!();

    // Split into two temp files
    let left_path = generate_temp_filename("merge_left");
    let right_path = generate_temp_filename("merge_right");

    let mut left = File::create(&left_path)?;
    let mut right = File::create(&right_path)?;

    let mid = data.len() / 2;
    for &num in &data[..mid] {
        writeln!(left, "{}", num)?;
    }
    for &num in &data[mid..] {
        writeln!(right, "{}", num)?;
    }

    println!("Split into two temp files:");
    println!("  Left file: {} elements", mid);
    println!("  Right file: {} elements", data.len() - mid);

    drop(left);
    drop(right);

    // Read back and display
    let left = File::open(&left_path)?;
    let right = File::open(&right_path)?;

    print!("Left half: ");
    for line in BufReader::new(left).lines() {
        print!("{} ", line?);
    }
    println!();

    print!("Right half: ");
    for line in BufReader::new(right).lines() {
        print!("{} ", line?);
    }
    println!();

    fs::remove_file(&left_path)?;
    fs::remove_file(&right_path)?;
    println!();

    Ok(())
}

// Temporary work file for text processing
fn demo_text_processing_temp() -> io::Result<()> {
    println!("=== Text processing with temp file ===");

    let input_lines = vec![
        "Hello, World!",
        "This is a test.",
        "Temporary files are useful.",
        "For processing data.",
        "End of input.",
    ];

    let temp_path = generate_temp_filename("text");
    let mut tmp = File::create(&temp_path)?;

    // Write input
    println!("Processing {} lines...", input_lines.len());
    for line in &input_lines {
        writeln!(tmp, "{}", line)?;
    }

    drop(tmp);

    // Read back and count words
    let tmp = File::open(&temp_path)?;
    let reader = BufReader::new(tmp);

    let mut total_words = 0;
    for line in reader.lines() {
        let line = line?;
        let words = line.split_whitespace().count();
        total_words += words;
    }

    println!("Total words: {}", total_words);

    fs::remove_file(&temp_path)?;

    Ok(())
}

// Note: For production use, consider the `tempfile` crate:
//
// use tempfile::{tempfile, NamedTempFile, TempDir};
//
// // Anonymous temp file (auto-deleted on drop)
// let mut tmp = tempfile()?;
// writeln!(tmp, "data")?;
//
// // Named temp file (auto-deleted on drop)
// let tmp = NamedTempFile::new()?;
// let path = tmp.path();
//
// // Temp directory (auto-deleted on drop)
// let dir = TempDir::new()?;

// Key differences from C:
// 1. tempfile crate for production (auto-cleanup)
// 2. std::env::temp_dir() for temp directory
// 3. RAII: temp files deleted on drop
// 4. No tmpfile/tmpnam functions
// 5. Type-safe file operations
// 6. Result<T, E> for error handling
// 7. BufReader for efficient reading
// 8. Iterator methods (lines, filter_map)
