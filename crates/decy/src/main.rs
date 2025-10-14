//! CLI tool for C-to-Rust transpilation with EXTREME quality standards.

#![warn(clippy::all)]
#![deny(unsafe_code)]

mod repl;

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
    /// Start interactive REPL mode
    Repl,
    /// Audit unsafe code in Rust files
    Audit {
        /// Path to the Rust source file to audit
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Show detailed information for each unsafe block
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Transpile { input, output }) => {
            transpile_file(input, output)?;
        }
        Some(Commands::Repl) => {
            repl::run()?;
        }
        Some(Commands::Audit { input, verbose }) => {
            audit_file(input, verbose)?;
        }
        None => {
            // No subcommand - show info
            println!("Decy: C-to-Rust Transpiler with EXTREME Quality Standards");
            println!("Version 0.1.0");
            println!();
            println!("Use 'decy --help' for usage information");
            println!("Use 'decy transpile <file>' to transpile C code to Rust");
            println!("Use 'decy repl' to start interactive mode");
            println!("Use 'decy audit <file>' to audit unsafe code in Rust files");
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

    // DECY-AUDIT-002: Detect if the source has no main function and provide guidance
    let has_main = rust_code.contains("fn main(");

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

            // DECY-AUDIT-002: Provide compilation guidance for library-only files
            if !has_main {
                eprintln!();
                eprintln!("ℹ Note: No main function found in source.");
                eprintln!("  To compile the output as a library, use:");
                eprintln!("  rustc --crate-type=lib {}", output_path.display());
            }
        }
        None => {
            // Write to stdout
            print!("{}", rust_code);

            // DECY-AUDIT-002: Provide compilation guidance for library-only files
            // Only show this to stderr if writing to stdout
            if !has_main {
                eprintln!();
                eprintln!("ℹ Note: No main function found in source.");
                eprintln!("  To compile the output as a library, use:");
                eprintln!("  rustc --crate-type=lib <output_file>");
            }
        }
    }

    Ok(())
}

fn audit_file(input: PathBuf, verbose: bool) -> Result<()> {
    // Read Rust file
    let rust_code = fs::read_to_string(&input)
        .with_context(|| format!("Failed to read input file: {}", input.display()))?;

    // Run unsafe code auditor
    let report = decy_verify::audit_rust_code(&rust_code)
        .with_context(|| format!("Failed to audit {}", input.display()))?;

    // Print header
    println!();
    println!("Unsafe Code Audit Report");
    println!("========================");
    println!("File: {}", input.display());
    println!("Total Lines: {}", report.total_lines);
    println!("Unsafe Lines: {}", report.unsafe_lines);
    println!(
        "Unsafe Density: {:.2}% {}",
        report.unsafe_density_percent,
        if report.meets_density_target() {
            "✅ (Target: <5%)"
        } else {
            "❌ (Target: <5%)"
        }
    );
    println!();

    if report.unsafe_blocks.is_empty() {
        println!("✅ No unsafe blocks found - code is 100% safe!");
        return Ok(());
    }

    println!("Unsafe Blocks Found: {}", report.unsafe_blocks.len());
    println!("Average Confidence: {:.1}/100", report.average_confidence);
    println!();

    // Show high-confidence blocks
    let high_conf = report.high_confidence_blocks();
    if !high_conf.is_empty() {
        println!(
            "⚠️  {} blocks with HIGH confidence for elimination (≥70):",
            high_conf.len()
        );
        println!();
    }

    // List all unsafe blocks
    if verbose {
        println!("Detailed Block Analysis:");
        println!("------------------------");
        for (idx, block) in report.unsafe_blocks.iter().enumerate() {
            println!();
            println!(
                "{}. Line {} [Confidence: {}/100 - {}]",
                idx + 1,
                if block.line > 0 {
                    block.line.to_string()
                } else {
                    "N/A".to_string()
                },
                block.confidence,
                if block.confidence >= 70 {
                    "HIGH"
                } else if block.confidence >= 40 {
                    "MEDIUM"
                } else {
                    "LOW"
                }
            );
            println!("   Pattern: {:?}", block.pattern);
            println!("   Suggestion: {}", block.suggestion);
        }
    } else {
        println!("Summary by Confidence:");
        let high = report
            .unsafe_blocks
            .iter()
            .filter(|b| b.confidence >= 70)
            .count();
        let medium = report
            .unsafe_blocks
            .iter()
            .filter(|b| b.confidence >= 40 && b.confidence < 70)
            .count();
        let low = report
            .unsafe_blocks
            .iter()
            .filter(|b| b.confidence < 40)
            .count();

        println!("  HIGH (≥70):   {} blocks - likely can be eliminated", high);
        println!(
            "  MEDIUM (40-69): {} blocks - review possible alternatives",
            medium
        );
        println!("  LOW (<40):    {} blocks - may require unsafe", low);
        println!();
        println!("Use --verbose flag to see detailed information for each block");
    }

    println!();
    println!("---");
    println!("Recommendation: Focus on eliminating HIGH confidence blocks first.");
    println!();

    Ok(())
}
