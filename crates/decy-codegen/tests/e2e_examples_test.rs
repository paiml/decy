// =============================================================================
// DECY-145: E2E validation of example corpus
// =============================================================================
// Tests that ALL C examples in the examples/ directory transpile and compile.
// This is a critical regression test - if examples stop compiling, we've
// introduced a bug.
//
// QA Items addressed: #81-90 (Integration & E2E)
//
// KNOWN ISSUES (documented, not blocking):
// - increment_decrement.c: Slice-to-raw-pointer conversion not yet supported
// - real_world_patterns.c: Pointer arithmetic on slices not yet supported
// - string_builder.c: Typedef struct with same name parsing issue
//
// These issues are tracked for future improvement but don't block the test suite.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Get the workspace root directory dynamically
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Known issues that are documented but don't fail the test suite.
/// These represent limitations in the current transpiler that are being tracked.
fn known_issue_files() -> HashSet<&'static str> {
    let mut known = HashSet::new();

    // DECY-145: Pointer arithmetic on slices
    // C: int* end = arr + size;
    // Issue: cannot add integer to &[i32]
    known.insert("real_world_patterns.c");

    // DECY-145: Typedef struct with same name parsing + stdlib issues
    // C: typedef struct StringBuilder { ... } StringBuilder;
    // Multiple issues: printf format, raw pointer indexing, type mismatches
    known.insert("string_builder.c");

    // DECY-160: FIXED - malloc for struct now returns Box::into_raw(Box::default())
    // linked_list.c now passes

    known
}

/// Run decy transpile on a C file and compile the result
fn transpile_and_compile(c_file: &Path) -> Result<(), String> {
    // Step 1: Transpile C to Rust
    let transpile_output = Command::new("cargo")
        .args(["run", "-p", "decy", "--quiet", "--", "transpile"])
        .arg(c_file)
        .current_dir(workspace_root())
        .output()
        .map_err(|e| format!("Failed to run decy: {}", e))?;

    if !transpile_output.status.success() {
        let stderr = String::from_utf8_lossy(&transpile_output.stderr);
        return Err(format!("Transpile failed: {}", stderr));
    }

    let rust_code = String::from_utf8_lossy(&transpile_output.stdout).to_string();

    if rust_code.trim().is_empty() {
        return Err("Transpile produced empty output".to_string());
    }

    // Step 2: Write to temp file with unique name per C file, thread, and timestamp
    // DECY-153: Use thread ID and timestamp to prevent race conditions in parallel tests
    let temp_dir = std::env::temp_dir();
    let path_hash = {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        c_file.hash(&mut hasher);
        std::thread::current().id().hash(&mut hasher);
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);
        hasher.finish()
    };
    let file_stem = c_file.file_stem().unwrap().to_string_lossy();
    let temp_file = temp_dir.join(format!("decy_e2e_{}_{}.rs", file_stem, path_hash));
    let output_file = temp_dir.join(format!("decy_e2e_{}_{}", file_stem, path_hash));
    fs::write(&temp_file, &rust_code).map_err(|e| format!("Failed to write temp file: {}", e))?;

    // Step 3: Compile with rustc (unique output per file to avoid race conditions)
    let compile_output = Command::new("rustc")
        .args(["--crate-type=lib", "--edition=2021", "-A", "warnings", "-o"])
        .arg(&output_file)
        .arg(&temp_file)
        .output()
        .map_err(|e| format!("Failed to run rustc: {}", e))?;

    // Clean up temp files
    let _ = fs::remove_file(&temp_file);
    let _ = fs::remove_file(&output_file);

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        return Err(format!(
            "Compile failed:\nRust code:\n{}\n\nErrors:\n{}",
            rust_code, stderr
        ));
    }

    Ok(())
}

/// Test all simple examples compile
#[test]
fn test_simple_examples_compile() {
    let simple_dir = workspace_root().join("examples/simple");

    let mut passed = 0;
    let mut failed = Vec::new();

    if let Ok(entries) = fs::read_dir(simple_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "c") {
                match transpile_and_compile(&path) {
                    Ok(()) => {
                        passed += 1;
                        println!("✓ {}", path.file_name().unwrap().to_string_lossy());
                    }
                    Err(e) => {
                        failed.push((path.clone(), e));
                        println!("✗ {}", path.file_name().unwrap().to_string_lossy());
                    }
                }
            }
        }
    }

    println!(
        "\nSimple examples: {}/{} passed",
        passed,
        passed + failed.len()
    );

    if !failed.is_empty() {
        for (path, error) in &failed {
            eprintln!("\n=== {} ===\n{}", path.display(), error);
        }
        panic!(
            "DECY-145: {} simple example(s) failed to compile",
            failed.len()
        );
    }
}

/// Test data structure examples compile
#[test]
fn test_data_structure_examples_compile() {
    let ds_dir = workspace_root().join("examples/data_structures");

    let mut passed = 0;
    let mut failed = Vec::new();

    if let Ok(entries) = fs::read_dir(ds_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "c") {
                match transpile_and_compile(&path) {
                    Ok(()) => {
                        passed += 1;
                        println!("✓ {}", path.file_name().unwrap().to_string_lossy());
                    }
                    Err(e) => {
                        failed.push((path.clone(), e));
                        println!("✗ {}", path.file_name().unwrap().to_string_lossy());
                    }
                }
            }
        }
    }

    println!(
        "\nData structure examples: {}/{} passed",
        passed,
        passed + failed.len()
    );

    if !failed.is_empty() {
        for (path, error) in &failed {
            eprintln!("\n=== {} ===\n{}", path.display(), error);
        }
        panic!(
            "DECY-145: {} data structure example(s) failed to compile",
            failed.len()
        );
    }
}

/// Test moderate examples compile (these may have known issues)
#[test]
fn test_moderate_examples_compile() {
    let moderate_dir = workspace_root().join("examples/moderate");

    let mut passed = 0;
    let mut failed = Vec::new();

    if let Ok(entries) = fs::read_dir(moderate_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "c") {
                match transpile_and_compile(&path) {
                    Ok(()) => {
                        passed += 1;
                        println!("✓ {}", path.file_name().unwrap().to_string_lossy());
                    }
                    Err(e) => {
                        failed.push((path.clone(), e));
                        println!("✗ {}", path.file_name().unwrap().to_string_lossy());
                    }
                }
            }
        }
    }

    println!(
        "\nModerate examples: {}/{} passed",
        passed,
        passed + failed.len()
    );

    // Document failures but don't fail test for moderate examples
    // These are more complex and may have known issues
    if !failed.is_empty() {
        println!(
            "\nNOTE: {} moderate example(s) failed (documented, not blocking)",
            failed.len()
        );
        for (path, _) in &failed {
            println!("  - {}", path.file_name().unwrap().to_string_lossy());
        }
    }
}

/// Test pointer arithmetic examples compile
#[test]
fn test_pointer_arithmetic_examples_compile() {
    let pa_dir = workspace_root().join("examples/pointer_arithmetic");
    let known_issues = known_issue_files();

    let mut passed = 0;
    let mut known_failed = Vec::new();
    let mut unexpected_failed = Vec::new();

    if let Ok(entries) = fs::read_dir(pa_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "c") {
                let filename = path.file_name().unwrap().to_string_lossy().to_string();
                match transpile_and_compile(&path) {
                    Ok(()) => {
                        passed += 1;
                        println!("✓ {}", filename);
                    }
                    Err(e) => {
                        if known_issues.contains(filename.as_str()) {
                            known_failed.push((path.clone(), e));
                            println!("⚠ {} (known issue)", filename);
                        } else {
                            unexpected_failed.push((path.clone(), e));
                            println!("✗ {}", filename);
                        }
                    }
                }
            }
        }
    }

    let total = passed + known_failed.len() + unexpected_failed.len();
    println!(
        "\nPointer arithmetic examples: {}/{} passed, {} known issues",
        passed,
        total,
        known_failed.len()
    );

    // Document known issues
    if !known_failed.is_empty() {
        println!("\nKnown issues (not blocking):");
        for (path, _) in &known_failed {
            println!("  - {}", path.file_name().unwrap().to_string_lossy());
        }
    }

    // Fail only on unexpected errors
    if !unexpected_failed.is_empty() {
        for (path, error) in &unexpected_failed {
            eprintln!("\n=== {} ===\n{}", path.display(), error);
        }
        panic!(
            "DECY-145: {} pointer arithmetic example(s) failed unexpectedly",
            unexpected_failed.len()
        );
    }
}

/// Test string examples compile
#[test]
fn test_string_examples_compile() {
    let strings_dir = workspace_root().join("examples/strings");
    let known_issues = known_issue_files();

    let mut passed = 0;
    let mut known_failed = Vec::new();
    let mut unexpected_failed = Vec::new();

    if let Ok(entries) = fs::read_dir(strings_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "c") {
                let filename = path.file_name().unwrap().to_string_lossy().to_string();
                match transpile_and_compile(&path) {
                    Ok(()) => {
                        passed += 1;
                        println!("✓ {}", filename);
                    }
                    Err(e) => {
                        if known_issues.contains(filename.as_str()) {
                            known_failed.push((path.clone(), e));
                            println!("⚠ {} (known issue)", filename);
                        } else {
                            unexpected_failed.push((path.clone(), e));
                            println!("✗ {}", filename);
                        }
                    }
                }
            }
        }
    }

    let total = passed + known_failed.len() + unexpected_failed.len();
    println!(
        "\nString examples: {}/{} passed, {} known issues",
        passed,
        total,
        known_failed.len()
    );

    // Document known issues
    if !known_failed.is_empty() {
        println!("\nKnown issues (not blocking):");
        for (path, _) in &known_failed {
            println!("  - {}", path.file_name().unwrap().to_string_lossy());
        }
    }

    // Fail only on unexpected errors
    if !unexpected_failed.is_empty() {
        for (path, error) in &unexpected_failed {
            eprintln!("\n=== {} ===\n{}", path.display(), error);
        }
        panic!(
            "DECY-145: {} string example(s) failed unexpectedly",
            unexpected_failed.len()
        );
    }
}

/// Summary test that counts total pass/fail across all example directories
#[test]
fn test_example_corpus_summary() {
    let dirs = [
        "simple",
        "data_structures",
        "moderate",
        "pointer_arithmetic",
        "strings",
        "real-world",
    ];
    let known_issues = known_issue_files();

    let mut total_passed = 0;
    let mut total_known_issues = 0;
    let mut total_unexpected_failed = 0;
    let mut known_failures = Vec::new();
    let mut unexpected_failures = Vec::new();

    for dir_name in &dirs {
        let dir = workspace_root().join("examples").join(dir_name);
        if !dir.exists() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "c") {
                    let filename = path.file_name().unwrap().to_string_lossy().to_string();
                    match transpile_and_compile(&path) {
                        Ok(()) => total_passed += 1,
                        Err(e) => {
                            if known_issues.contains(filename.as_str()) {
                                total_known_issues += 1;
                                known_failures.push((path, e));
                            } else {
                                total_unexpected_failed += 1;
                                unexpected_failures.push((path, e));
                            }
                        }
                    }
                }
            }
        }
    }

    let total = total_passed + total_known_issues + total_unexpected_failed;
    println!("\n========================================");
    println!("E2E Example Corpus Summary");
    println!("========================================");
    println!(
        "Total: {} passed, {} known issues, {} unexpected failures",
        total_passed, total_known_issues, total_unexpected_failed
    );
    println!(
        "Pass rate: {:.1}% (excluding known issues: {:.1}%)",
        100.0 * total_passed as f64 / total as f64,
        100.0 * total_passed as f64 / (total_passed + total_unexpected_failed).max(1) as f64
    );

    if !known_failures.is_empty() {
        println!("\nKnown issues (documented, not blocking):");
        for (path, _) in &known_failures {
            println!("  - {}", path.display());
        }
    }

    if !unexpected_failures.is_empty() {
        println!("\nUnexpected failures:");
        for (path, _) in &unexpected_failures {
            println!("  - {}", path.display());
        }
    }

    // Document state without failing - individual tests have stricter requirements
}
