/* K&R C Chapter 8.3: Open, Creat, Close, Unlink
 * Page 172-173
 * File creation and manipulation
 * Transpiled to safe Rust using std::fs
 */

use std::fs::{File, OpenOptions, remove_file};
use std::io::Write;

fn main() -> std::io::Result<()> {
    let message = b"Hello from creat!\n";

    // Create new file (or truncate existing)
    let mut fd = File::create("newfile.txt")?;

    // Write to file
    fd.write_all(message)?;
    drop(fd);  // Close

    println!("Created and wrote to newfile.txt");

    // Open existing file for append
    let mut fd = OpenOptions::new()
        .write(true)
        .append(true)
        .open("newfile.txt")?;

    fd.write_all(b"Appended line\n")?;
    drop(fd);  // Close

    println!("Appended to newfile.txt");

    // Remove file
    remove_file("newfile.txt")?;
    println!("Deleted newfile.txt");

    Ok(())
}
