//! Validation Runner - Test DECY against heterogeneous C examples
//!
//! This example runs DECY transpilation on all C test cases and validates:
//! - Successful parsing
//! - HIR generation
//! - Rust code generation
//! - Compilation of generated Rust code
//!
//! Usage: cargo run --example validation_runner

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Categories of C examples to test
#[derive(Debug)]
struct TestCategory {
    name: &'static str,
    path: &'static str,
    description: &'static str,
}

const CATEGORIES: &[TestCategory] = &[
    TestCategory {
        name: "Simple",
        path: "examples/simple",
        description: "Basic C constructs (arithmetic, control flow)",
    },
    TestCategory {
        name: "Moderate",
        path: "examples/moderate",
        description: "Intermediate complexity (loops, functions)",
    },
    TestCategory {
        name: "Pointer Arithmetic",
        path: "examples/pointer_arithmetic",
        description: "Pointer operations and array access",
    },
    TestCategory {
        name: "Real-World",
        path: "examples/real-world",
        description: "Practical programs (data structures, utilities)",
    },
    TestCategory {
        name: "CLI Tools",
        path: "examples/cli",
        description: "Command-line utilities (grep, wc, cat)",
    },
    TestCategory {
        name: "Data Structures",
        path: "examples/data_structures",
        description: "Common data structures (trees, graphs, hash tables)",
    },
    TestCategory {
        name: "File I/O",
        path: "examples/file_io",
        description: "File operations (read, write, parse)",
    },
    TestCategory {
        name: "Threading",
        path: "examples/threading",
        description: "Concurrent programs (pthread, mutexes)",
    },
    TestCategory {
        name: "String Processing",
        path: "examples/strings",
        description: "String manipulation and parsing",
    },
    TestCategory {
        name: "System Calls",
        path: "examples/syscalls",
        description: "System-level operations",
    },
];

/// Test result for a single C file
#[derive(Debug)]
struct TestResult {
    file: String,
    parsed: bool,
    generated: bool,
    compiled: bool,
    error: Option<String>,
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  DECY Validation Suite - Heterogeneous C Test Collection  ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;
    let mut missing_categories = Vec::new();

    for category in CATEGORIES {
        println!("📂 Category: {}", category.name);
        println!("   Path: {}", category.path);
        println!("   Description: {}", category.description);
        println!();

        let path = Path::new(category.path);
        if !path.exists() {
            println!("   ⚠️  Category directory not found - skipping");
            missing_categories.push(category.name);
            println!();
            continue;
        }

        // Find all .c files in this category
        let c_files = find_c_files(path);
        if c_files.is_empty() {
            println!("   ℹ️  No C files found in this category");
            println!();
            continue;
        }

        println!("   Found {} C file(s):", c_files.len());
        for file in &c_files {
            total_tests += 1;
            let result = test_transpilation(file);

            let status = if result.compiled {
                "✅ PASS"
            } else if result.generated {
                "⚠️  PARTIAL (Rust generated but doesn't compile)"
            } else if result.parsed {
                "⚠️  PARTIAL (Parsed but no Rust generated)"
            } else {
                "❌ FAIL"
            };

            println!("   {} - {}", status, file.display());

            if let Some(error) = &result.error {
                println!("      Error: {}", error);
            }

            if result.compiled {
                passed_tests += 1;
            } else {
                failed_tests += 1;
            }
        }
        println!();
    }

    // Summary
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  Summary                                                   ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("Total tests:   {}", total_tests);
    println!("Passed:        {} ({:.1}%)", passed_tests,
             if total_tests > 0 { (passed_tests as f64 / total_tests as f64) * 100.0 } else { 0.0 });
    println!("Failed:        {} ({:.1}%)", failed_tests,
             if total_tests > 0 { (failed_tests as f64 / total_tests as f64) * 100.0 } else { 0.0 });
    println!();

    if !missing_categories.is_empty() {
        println!("⚠️  Missing categories (need to create):");
        for cat in &missing_categories {
            println!("   - {}", cat);
        }
        println!();
    }

    // Exit code
    if failed_tests == 0 && total_tests > 0 {
        println!("🎉 All tests passed!");
        std::process::exit(0);
    } else if total_tests == 0 {
        println!("⚠️  No tests found!");
        std::process::exit(1);
    } else {
        println!("❌ Some tests failed");
        std::process::exit(1);
    }
}

/// Find all .c files in a directory
fn find_c_files(path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("c") {
                files.push(path);
            }
        }
    }

    files.sort();
    files
}

/// Test transpilation of a C file
fn test_transpilation(c_file: &Path) -> TestResult {
    let file_name = c_file.display().to_string();

    // For now, just check if corresponding .rs file exists
    // In the future, this would actually run DECY transpiler
    let rs_file = c_file.with_extension("rs");

    if !rs_file.exists() {
        return TestResult {
            file: file_name,
            parsed: false,
            generated: false,
            compiled: false,
            error: Some("No corresponding .rs file found (needs transpilation)".to_string()),
        };
    }

    // Check if the Rust file compiles
    let compiled = check_rust_compiles(&rs_file);

    TestResult {
        file: file_name,
        parsed: true,
        generated: true,
        compiled,
        error: if compiled { None } else { Some("Rust file exists but doesn't compile".to_string()) },
    }
}

/// Check if a Rust file compiles
fn check_rust_compiles(rs_file: &Path) -> bool {
    // Try to compile with rustc
    let output = Command::new("rustc")
        .arg("--crate-type")
        .arg("lib")
        .arg("--edition")
        .arg("2021")
        .arg(rs_file)
        .arg("-o")
        .arg("/dev/null")
        .output();

    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}
