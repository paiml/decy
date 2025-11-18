// Simple CSV parser
// Transpiled to safe Rust using Vec<Record> and std::fs

use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct Record {
    name: String,
    age: i32,
    city: String,
}

fn parse_csv_line(line: &str) -> Result<Record, String> {
    let parts: Vec<&str> = line.trim().split(',').collect();

    if parts.len() < 3 {
        return Err("Invalid CSV line: not enough fields".to_string());
    }

    let name = parts[0].to_string();
    let age = parts[1].trim().parse::<i32>()
        .map_err(|_| "Invalid age field")?;
    let city = parts[2].to_string();

    Ok(Record { name, age, city })
}

fn read_csv(filename: &str) -> Result<Vec<Record>, std::io::Error> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    let mut records: Vec<Record> = Vec::with_capacity(10);

    for line_result in reader.lines() {
        let line = line_result?;
        if let Ok(record) = parse_csv_line(&line) {
            records.push(record);
        }
    }

    Ok(records)
}

fn main() {
    // Note: This would need a test CSV file to actually run
    // For now, just demonstrate the API
    println!("CSV Parser example");
    println!("Would parse: name,age,city format");

    // Example usage:
    // match read_csv("data.csv") {
    //     Ok(records) => {
    //         for record in records {
    //             println!("{:?}", record);
    //         }
    //     }
    //     Err(e) => eprintln!("Error: {}", e),
    // }
}
