// Simple word count utility (like wc -w)
// Transpiled to safe Rust using std::fs and std::io

use std::fs::File;
use std::io::{BufRead, BufReader};

fn count_words(filename: &str) -> Result<i32, std::io::Error> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    let mut word_count: i32 = 0;
    let mut in_word: bool = false;

    for line_result in reader.lines() {
        let line = line_result?;
        for ch in line.chars() {
            if ch.is_whitespace() {
                in_word = false;
            } else if !in_word {
                in_word = true;
                word_count += 1;
            }
        }
    }

    Ok(word_count)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    match count_words(&args[1]) {
        Ok(count) => {
            println!("{}", count);
            Ok(())
        }
        Err(_) => {
            eprintln!("Error: Could not open file {}", args[1]);
            std::process::exit(1);
        }
    }
}
