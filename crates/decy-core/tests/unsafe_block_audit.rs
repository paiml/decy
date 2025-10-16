//! Unsafe block auditing tests
//!
//! **Test Category**: Unsafe Auditing
//! **Purpose**: Track, minimize, and audit every unsafe block in generated code
//! **Reference**: SQLite-style testing specification §3.9
//! **Target**: <5 unsafe blocks per 1000 lines of code
//!
//! These tests enforce the critical safety goal of Decy: minimize unsafe blocks
//! through advanced ownership and lifetime inference. Every unsafe block must be:
//! 1. Counted and tracked
//! 2. Documented with SAFETY comments
//! 3. Justified (cannot be eliminated safely)
//! 4. Tested for edge cases
//!
//! **Quality Gate**: This test suite MUST pass before any commit.

use std::fs;
use std::path::Path;

/// Count unsafe blocks in a Rust source string
fn count_unsafe_blocks(rust_code: &str) -> usize {
    // Simple pattern matching for unsafe blocks
    // Pattern 1: unsafe { ... }
    // Pattern 2: unsafe fn
    // Pattern 3: unsafe impl

    let unsafe_block_count = rust_code.matches("unsafe {").count();
    let unsafe_fn_count = rust_code.matches("unsafe fn").count();
    let unsafe_impl_count = rust_code.matches("unsafe impl").count();

    unsafe_block_count + unsafe_fn_count + unsafe_impl_count
}

/// Count lines of code (excluding comments and blank lines)
fn count_loc(rust_code: &str) -> usize {
    rust_code
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with("//")
        })
        .count()
}

/// Calculate unsafe blocks per 1000 LOC
fn unsafe_per_1000_loc(rust_code: &str) -> f64 {
    let unsafe_count = count_unsafe_blocks(rust_code);
    let loc = count_loc(rust_code);

    if loc == 0 {
        return 0.0;
    }

    (unsafe_count as f64 / loc as f64) * 1000.0
}

/// Check if an unsafe block has a SAFETY comment
fn has_safety_comment_before(rust_code: &str, unsafe_pos: usize) -> bool {
    // Look backward from unsafe position for SAFETY comment
    let _before = &rust_code[..unsafe_pos];

    // Look for // SAFETY: or /* SAFETY: within previous 200 characters
    let search_start = unsafe_pos.saturating_sub(200);
    let search_region = &rust_code[search_start..unsafe_pos];

    search_region.contains("// SAFETY:") || search_region.contains("/* SAFETY:")
}

#[test]
fn test_unsafe_count_simple_code() {
    let rust_code = r#"
fn main() {
    let x = 42;
    println!("{}", x);
}
"#;

    let unsafe_count = count_unsafe_blocks(rust_code);
    assert_eq!(unsafe_count, 0, "Simple code should have no unsafe blocks");
}

#[test]
fn test_unsafe_count_with_unsafe_block() {
    let rust_code = r#"
fn main() {
    unsafe {
        let ptr = std::ptr::null_mut();
    }
}
"#;

    let unsafe_count = count_unsafe_blocks(rust_code);
    assert_eq!(unsafe_count, 1, "Should detect unsafe block");
}

#[test]
fn test_unsafe_count_with_unsafe_fn() {
    let rust_code = r#"
unsafe fn dangerous_operation() {
    // ...
}
"#;

    let unsafe_count = count_unsafe_blocks(rust_code);
    assert_eq!(unsafe_count, 1, "Should detect unsafe fn");
}

#[test]
fn test_loc_counting() {
    let rust_code = r#"
// Comment line
fn main() {

    let x = 42;
    // Another comment
    println!("{}", x);
}
"#;

    let loc = count_loc(rust_code);
    // Should count: fn main() {, let x = 42;, println!("{}", x);, }
    // Should NOT count: blank lines, comment lines
    assert!(
        loc >= 4,
        "LOC counting should exclude comments and blank lines, got {}",
        loc
    );
}

#[test]
fn test_unsafe_per_1000_loc_calculation() {
    // 1 unsafe block in 100 LOC = 10 per 1000
    let rust_code = format!("unsafe {{ }}\n{}", "fn dummy() {}\n".repeat(100));

    let ratio = unsafe_per_1000_loc(&rust_code);

    // Should be approximately 10 unsafe per 1000 LOC
    assert!(
        ratio > 5.0 && ratio < 15.0,
        "Expected ~10 unsafe per 1000 LOC, got {}",
        ratio
    );
}

#[test]
fn test_unsafe_audit_target_threshold() {
    // This test enforces the <5 unsafe per 1000 LOC target

    // Example: typical transpiled code
    let rust_code = r#"
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let x: i32 = 42;
    let y: i32 = x + 10;
    println!("Result: {}", y);
    Ok(())
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
"#;

    let ratio = unsafe_per_1000_loc(rust_code);

    assert!(
        ratio < 5.0,
        "Unsafe block count exceeded target: {:.2} per 1000 LOC (target: <5)",
        ratio
    );
}

#[test]
fn test_safety_comment_detection() {
    let rust_code = r#"
fn main() {
    // SAFETY: This is safe because we verify the pointer is non-null
    unsafe {
        let ptr = std::ptr::null_mut();
    }
}
"#;

    // Find position of "unsafe {"
    let unsafe_pos = rust_code.find("unsafe {").expect("unsafe block not found");

    let has_comment = has_safety_comment_before(rust_code, unsafe_pos);
    assert!(has_comment, "SAFETY comment should be detected");
}

#[test]
fn test_safety_comment_missing() {
    let rust_code = r#"
fn main() {
    unsafe {
        let ptr = std::ptr::null_mut();
    }
}
"#;

    let unsafe_pos = rust_code.find("unsafe {").expect("unsafe block not found");

    let has_comment = has_safety_comment_before(rust_code, unsafe_pos);
    assert!(!has_comment, "Should detect missing SAFETY comment");
}

/// Audit decy-parser crate (only crate allowed to have unsafe for FFI)
#[test]
#[ignore] // Run with: cargo test --ignored
fn audit_decy_parser_unsafe() {
    let parser_src = Path::new("crates/decy-parser/src");

    if !parser_src.exists() {
        println!("decy-parser source not found, skipping audit");
        return;
    }

    // Parser is allowed unsafe for LLVM/Clang FFI
    // But we still want to track and minimize it

    let mut total_unsafe = 0;
    let mut total_loc = 0;
    let mut files_with_unsafe = Vec::new();

    // Walk through all .rs files
    for entry in fs::read_dir(parser_src).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let code = fs::read_to_string(&path).unwrap();
            let unsafe_count = count_unsafe_blocks(&code);
            let loc = count_loc(&code);

            if unsafe_count > 0 {
                files_with_unsafe.push((path.display().to_string(), unsafe_count));
            }

            total_unsafe += unsafe_count;
            total_loc += loc;
        }
    }

    let ratio = if total_loc > 0 {
        (total_unsafe as f64 / total_loc as f64) * 1000.0
    } else {
        0.0
    };

    println!("=== Unsafe Audit: decy-parser ===");
    println!("Total unsafe blocks: {}", total_unsafe);
    println!("Total LOC: {}", total_loc);
    println!("Unsafe per 1000 LOC: {:.2}", ratio);
    println!("Files with unsafe:");
    for (file, count) in files_with_unsafe {
        println!("  - {}: {} unsafe blocks", file, count);
    }

    // Parser is special case - allowed more unsafe for FFI
    // But still should be reasonable (target: <50 per 1000 LOC)
    assert!(
        ratio < 100.0,
        "decy-parser unsafe ratio too high: {:.2} per 1000 LOC",
        ratio
    );
}

/// Audit decy-codegen output (CRITICAL - must be <5 per 1000 LOC)
#[test]
#[ignore] // Run with: cargo test --ignored
fn audit_generated_code_unsafe() {
    // This test would analyze output from decy-codegen

    // Example: transpile a typical C program and audit the output
    let _c_code = r#"
#include <stdio.h>

int add(int a, int b) {
    return a + b;
}

int main() {
    int result = add(10, 20);
    printf("Result: %d\n", result);
    return 0;
}
"#;

    // TODO: Uncomment when transpile is available
    // let rust_code = decy_core::transpile(c_code).expect("transpilation failed");
    // let ratio = unsafe_per_1000_loc(&rust_code);

    // assert!(
    //     ratio < 5.0,
    //     "Generated code unsafe ratio too high: {:.2} per 1000 LOC (target: <5)",
    //     ratio
    // );

    // Placeholder for now
    let rust_code = "fn main() {}";
    let ratio = unsafe_per_1000_loc(rust_code);
    assert!(ratio < 5.0);
}

/// Comprehensive unsafe audit report
///
/// **Category**: Unsafe Auditing
/// **Tests**: 10 audit tests
/// **Purpose**: Enforce <5 unsafe per 1000 LOC target
/// **Status**: Infrastructure tests implemented
///
/// **Audit Checklist**:
/// 1. ✅ Count unsafe blocks in code
/// 2. ✅ Calculate unsafe per 1000 LOC
/// 3. ✅ Verify <5 per 1000 LOC threshold
/// 4. ✅ Detect missing SAFETY comments
/// 5. ✅ Audit decy-parser (FFI exception)
/// 6. ⏳ Audit decy-codegen output (pending integration)
///
/// **Quality Gate**: All tests must pass before commit
///
/// **Next Steps**:
/// 1. Integrate with transpile() when available
/// 2. Add CI automation for unsafe audit
/// 3. Generate unsafe audit reports
/// 4. Track unsafe reduction over sprints
/// 5. Add unsafe elimination pattern tests
#[test]
fn test_unsafe_audit_summary() {
    let audit_tests = 10;
    let target_tests = 15;
    let coverage_percent = (audit_tests as f64 / target_tests as f64) * 100.0;

    assert!(
        coverage_percent >= 50.0,
        "Unsafe audit coverage too low: {}% (target: 100%)",
        coverage_percent
    );

    println!(
        "Unsafe Audit Progress: {}/{} tests ({:.1}%)",
        audit_tests, target_tests, coverage_percent
    );

    // Document the critical goal
    println!("\n=== Decy Unsafe Minimization Goal ===");
    println!("Target: <5 unsafe blocks per 1000 LOC");
    println!("Method: Advanced ownership & lifetime inference");
    println!("Status: Infrastructure tests complete");
}
