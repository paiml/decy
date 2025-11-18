/* K&R C Chapter 7.5: File Access
 * Page 160-162
 * Basic file operations (fopen, fclose, fprintf, fscanf)
 * Transpiled to safe Rust using std::fs
 */

use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

fn main() -> io::Result<()> {
    // Write to file
    let mut fp = File::create("numbers.txt")?;

    for i in 1..=10 {
        writeln!(fp, "{}", i * i)?;
    }

    // File automatically closed when fp goes out of scope

    // Read from file
    let fp = File::open("numbers.txt")?;
    let reader = BufReader::new(fp);

    println!("Squares from file:");
    for line in reader.lines() {
        let line = line?;
        if let Ok(num) = line.parse::<i32>() {
            println!("{}", num);
        }
    }

    Ok(())
}
