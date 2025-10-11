//! CLI tool for C-to-Rust transpilation with EXTREME quality standards.

#![warn(clippy::all)]
#![deny(unsafe_code)]

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

/// Decy: C-to-Rust Transpiler with EXTREME Quality Standards
#[derive(Parser, Debug)]
#[command(name = "decy")]
#[command(version = "0.1.0")]
#[command(about = "Transpile C code to safe Rust with minimal unsafe blocks", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Transpile a C source file to Rust
    Transpile {
        /// Path to the C source file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file (default: stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Transpile { input, output }) => {
            transpile_file(input, output)?;
        }
        None => {
            // No subcommand - show info
            println!("Decy: C-to-Rust Transpiler with EXTREME Quality Standards");
            println!("Version 0.1.0");
            println!();
            println!("Use 'decy --help' for usage information");
            println!("Use 'decy transpile <file>' to transpile C code to Rust");
        }
    }

    Ok(())
}

fn transpile_file(input: PathBuf, output: Option<PathBuf>) -> Result<()> {
    // Read input file
    let c_code = fs::read_to_string(&input)
        .with_context(|| format!("Failed to read input file: {}", input.display()))?;

    // Transpile using decy-core
    let rust_code = decy_core::transpile(&c_code)
        .with_context(|| format!("Failed to transpile {}", input.display()))?;

    // Write output
    match output {
        Some(output_path) => {
            fs::write(&output_path, rust_code).with_context(|| {
                format!("Failed to write output file: {}", output_path.display())
            })?;
            eprintln!(
                "✓ Transpiled {} → {}",
                input.display(),
                output_path.display()
            );
        }
        None => {
            // Write to stdout
            print!("{}", rust_code);
        }
    }

    Ok(())
}
