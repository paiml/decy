// Simple grep-like utility
// Transpiled to safe Rust using std::fs and std::io

use std::fs::File;
use std::io::{BufRead, BufReader};

fn grep_file(pattern: &str, filename: &str) -> Result<i32, std::io::Error> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    let mut match_count: i32 = 0;

    for line_result in reader.lines() {
        let line = line_result?;
        if line.contains(pattern) {
            println!("{}", line);
            match_count += 1;
        }
    }

    Ok(match_count)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <pattern> <filename>", args[0]);
        std::process::exit(1);
    }

    match grep_file(&args[1], &args[2]) {
        Ok(matches) => {
            eprintln!("Found {} matching lines", matches);
            Ok(())
        }
        Err(_) => {
            eprintln!("Error: Could not open file {}", args[2]);
            std::process::exit(1);
        }
    }
}
