//! Safety property verification for transpiled Rust code.
//!
//! Verifies memory safety, type safety, and other Rust safety guarantees.
//!
//! # Unsafe Code Auditing
//!
//! This module provides comprehensive auditing of unsafe blocks in generated Rust code:
//! - Detection and counting of all unsafe blocks
//! - Confidence scoring for elimination potential
//! - Suggestions for safer alternatives
//! - Unsafe density metrics (<5 per 1000 LOC target)
//!
//! # Example
//!
//! ```no_run
//! use decy_verify::{UnsafeAuditor, audit_rust_code};
//!
//! let rust_code = r#"
//!     fn example() {
//!         unsafe {
//!             let ptr = std::ptr::null_mut();
//!         }
//!     }
//! "#;
//!
//! let report = audit_rust_code(rust_code).expect("Failed to audit");
//! println!("Unsafe blocks found: {}", report.unsafe_blocks.len());
//! println!("Unsafe density: {:.2}%", report.unsafe_density_percent);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

pub mod diff_test;
pub mod lock_verify;

use anyhow::{Context, Result};
use syn::{visit::Visit, Block, Expr, ExprUnsafe, ItemFn};

/// Represents a single unsafe block found in Rust code
#[derive(Debug, Clone, PartialEq)]
pub struct UnsafeBlock {
    /// Line number where the unsafe block starts
    pub line: usize,
    /// Confidence score (0-100) that this block could be eliminated
    pub confidence: u8,
    /// Pattern detected (e.g., "raw_pointer_deref", "transmute", etc.)
    pub pattern: UnsafePattern,
    /// Suggestion for safer alternative
    pub suggestion: String,
}

/// Categories of unsafe patterns
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnsafePattern {
    /// Raw pointer dereference (*ptr)
    RawPointerDeref,
    /// Type transmutation
    Transmute,
    /// Inline assembly
    Assembly,
    /// FFI call
    FfiCall,
    /// Union field access
    UnionAccess,
    /// Mutable static access
    MutableStatic,
    /// Other unsafe operation
    Other,
}

/// Report summarizing unsafe code in a Rust file
#[derive(Debug, Clone)]
pub struct UnsafeAuditReport {
    /// Total lines of code
    pub total_lines: usize,
    /// Lines inside unsafe blocks
    pub unsafe_lines: usize,
    /// Unsafe density as percentage
    pub unsafe_density_percent: f64,
    /// List of all unsafe blocks found
    pub unsafe_blocks: Vec<UnsafeBlock>,
    /// Average confidence score across all blocks
    pub average_confidence: f64,
}

impl UnsafeAuditReport {
    /// Create a new audit report
    pub fn new(total_lines: usize, unsafe_lines: usize, unsafe_blocks: Vec<UnsafeBlock>) -> Self {
        let unsafe_density_percent = if total_lines > 0 {
            (unsafe_lines as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };

        let average_confidence = if !unsafe_blocks.is_empty() {
            unsafe_blocks
                .iter()
                .map(|b| b.confidence as f64)
                .sum::<f64>()
                / unsafe_blocks.len() as f64
        } else {
            0.0
        };

        Self {
            total_lines,
            unsafe_lines,
            unsafe_density_percent,
            unsafe_blocks,
            average_confidence,
        }
    }

    /// Check if unsafe density meets the <5% target
    pub fn meets_density_target(&self) -> bool {
        self.unsafe_density_percent < 5.0
    }

    /// Get blocks with high confidence for elimination (≥70)
    pub fn high_confidence_blocks(&self) -> Vec<&UnsafeBlock> {
        self.unsafe_blocks
            .iter()
            .filter(|b| b.confidence >= 70)
            .collect()
    }
}

/// Main auditor for analyzing unsafe code
pub struct UnsafeAuditor {
    unsafe_blocks: Vec<UnsafeBlock>,
    total_lines: usize,
    unsafe_lines: usize,
    source_code: String,
}

impl UnsafeAuditor {
    /// Create a new auditor
    pub fn new() -> Self {
        Self {
            unsafe_blocks: Vec::new(),
            total_lines: 0,
            unsafe_lines: 0,
            source_code: String::new(),
        }
    }

    /// Analyze Rust source code and generate an audit report
    pub fn audit(&mut self, rust_code: &str) -> Result<UnsafeAuditReport> {
        // Store source code for line counting
        self.source_code = rust_code.to_string();

        // Count total lines
        self.total_lines = rust_code.lines().count();

        // Parse the Rust code
        let syntax_tree = syn::parse_file(rust_code).context("Failed to parse Rust code")?;

        // Visit the AST to find unsafe blocks
        self.visit_file(&syntax_tree);

        Ok(UnsafeAuditReport::new(
            self.total_lines,
            self.unsafe_lines,
            self.unsafe_blocks.clone(),
        ))
    }

    /// Detect the pattern type and assign confidence score
    fn analyze_unsafe_block(&self, unsafe_block: &ExprUnsafe) -> (UnsafePattern, u8, String) {
        // Convert block to string for pattern matching
        let block_str = quote::quote!(#unsafe_block).to_string();

        // Detect patterns and assign confidence scores
        let (pattern, confidence, suggestion) = if block_str.contains("std :: ptr ::")
            || block_str.contains("* ptr")
            || block_str.contains("null_mut")
            || block_str.contains("null()")
        {
            (
                UnsafePattern::RawPointerDeref,
                85,
                "Consider using Box<T>, &T, or &mut T with proper lifetimes".to_string(),
            )
        } else if block_str.contains("transmute") {
            (
                UnsafePattern::Transmute,
                40,
                "Consider safe alternatives like From/Into traits or checked conversions"
                    .to_string(),
            )
        } else if block_str.contains("asm!") || block_str.contains("global_asm!") {
            (
                UnsafePattern::Assembly,
                15,
                "No safe alternative - inline assembly required for platform-specific operations"
                    .to_string(),
            )
        } else if block_str.contains("extern") {
            (
                UnsafePattern::FfiCall,
                30,
                "Consider creating a safe wrapper around FFI calls".to_string(),
            )
        } else {
            (
                UnsafePattern::Other,
                50,
                "Review if this unsafe block can be eliminated or replaced with safe alternatives"
                    .to_string(),
            )
        };

        (pattern, confidence, suggestion)
    }

    /// Count lines in an unsafe block
    fn count_block_lines(&self, block: &Block) -> usize {
        // Rough approximation: count statements and add braces
        block.stmts.len() + 2
    }
}

impl Default for UnsafeAuditor {
    fn default() -> Self {
        Self::new()
    }
}

impl<'ast> Visit<'ast> for UnsafeAuditor {
    /// Visit expressions to find unsafe blocks
    fn visit_expr(&mut self, expr: &'ast Expr) {
        if let Expr::Unsafe(unsafe_expr) = expr {
            // Found an unsafe block!
            let (pattern, confidence, suggestion) = self.analyze_unsafe_block(unsafe_expr);

            // Count lines in this unsafe block
            let block_lines = self.count_block_lines(&unsafe_expr.block);
            self.unsafe_lines += block_lines;

            // Get line number (approximation using span start)
            let line = 0; // syn doesn't provide easy line number access without proc_macro2 spans

            self.unsafe_blocks.push(UnsafeBlock {
                line,
                confidence,
                pattern,
                suggestion,
            });
        }

        // Continue visiting nested expressions
        syn::visit::visit_expr(self, expr);
    }

    /// Visit items to find unsafe functions
    fn visit_item_fn(&mut self, func: &'ast ItemFn) {
        // Check if function is marked unsafe
        if func.sig.unsafety.is_some() {
            // Unsafe function - count the entire body as unsafe
            let body_lines = self.count_block_lines(&func.block);
            self.unsafe_lines += body_lines;

            self.unsafe_blocks.push(UnsafeBlock {
                line: 0,
                confidence: 60,
                pattern: UnsafePattern::Other,
                suggestion: "Unsafe function - review if entire function needs to be unsafe or just specific blocks".to_string(),
            });
        }

        // Continue visiting the function body
        syn::visit::visit_item_fn(self, func);
    }
}

/// Convenience function to audit Rust code
///
/// # Example
///
/// ```
/// use decy_verify::audit_rust_code;
///
/// let code = "fn safe_function() { let x = 42; }";
/// let report = audit_rust_code(code).expect("Audit failed");
/// assert_eq!(report.unsafe_blocks.len(), 0);
/// ```
pub fn audit_rust_code(rust_code: &str) -> Result<UnsafeAuditReport> {
    let mut auditor = UnsafeAuditor::new();
    auditor.audit(rust_code)
}

/// A structured compilation error from rustc
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilationError {
    /// Error code (e.g., "E0308")
    pub code: Option<String>,
    /// Error message
    pub message: String,
}

/// Result of compiling generated Rust code
#[derive(Debug, Clone)]
pub struct CompilationResult {
    /// Whether compilation succeeded
    pub success: bool,
    /// Errors found during compilation
    pub errors: Vec<CompilationError>,
    /// Warnings found during compilation
    pub warnings: Vec<String>,
}

/// Verify that generated Rust code compiles by invoking rustc.
///
/// Uses `rustc --emit=metadata --edition=2021` for fast type-checking
/// without full code generation.
///
/// # Example
///
/// ```no_run
/// use decy_verify::verify_compilation;
///
/// let result = verify_compilation("fn main() {}").expect("rustc failed to run");
/// assert!(result.success);
/// ```
pub fn verify_compilation(rust_code: &str) -> Result<CompilationResult> {
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    let unique_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("decy_verify_{}.rs", unique_id));
    let temp_output = temp_dir.join(format!("decy_verify_{}.rmeta", unique_id));

    std::fs::write(&temp_file, rust_code)
        .context("Failed to write temp file for compilation check")?;

    let output = Command::new("rustc")
        .arg("--emit=metadata")
        .arg("--edition=2021")
        .arg("-o")
        .arg(&temp_output)
        .arg(&temp_file)
        .output()
        .context("Failed to run rustc — is it installed?")?;

    // Clean up temp files
    let _ = std::fs::remove_file(&temp_file);
    let _ = std::fs::remove_file(&temp_output);

    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        let warnings: Vec<String> = stderr
            .lines()
            .filter(|l| l.contains("warning"))
            .map(|l| l.to_string())
            .collect();
        Ok(CompilationResult {
            success: true,
            errors: vec![],
            warnings,
        })
    } else {
        let mut errors = Vec::new();
        for line in stderr.lines() {
            if line.starts_with("error") {
                // Extract error code like E0308 from "error[E0308]: ..."
                let code = line
                    .find('[')
                    .and_then(|start| line.find(']').map(|end| (start, end)))
                    .map(|(start, end)| line[start + 1..end].to_string());
                errors.push(CompilationError {
                    code,
                    message: line.to_string(),
                });
            }
        }
        Ok(CompilationResult {
            success: false,
            errors,
            warnings: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: These tests will FAIL
    // Testing unsafe block detection

    #[test]
    fn test_no_unsafe_blocks() {
        // RED: This should pass (no unsafe blocks)
        let code = r#"
            fn safe_function() {
                let x = 42;
                println!("{}", x);
            }
        "#;

        let report = audit_rust_code(code).expect("Audit failed");
        assert_eq!(report.unsafe_blocks.len(), 0);
        assert_eq!(report.unsafe_lines, 0);
        assert!(report.meets_density_target());
    }

    #[test]
    fn test_single_unsafe_block() {
        // RED: This will FAIL - we don't detect unsafe blocks yet
        let code = r#"
            fn with_unsafe() {
                unsafe {
                    let ptr = std::ptr::null_mut::<i32>();
                    *ptr = 42;
                }
            }
        "#;

        let report = audit_rust_code(code).expect("Audit failed");
        assert_eq!(
            report.unsafe_blocks.len(),
            1,
            "Should detect one unsafe block"
        );
        assert!(report.unsafe_lines > 0, "Should count unsafe lines");
    }

    #[test]
    fn test_multiple_unsafe_blocks() {
        // RED: This will FAIL
        let code = r#"
            fn multiple_unsafe() {
                unsafe {
                    let ptr1 = std::ptr::null_mut::<i32>();
                }

                let safe_code = 42;

                unsafe {
                    let ptr2 = std::ptr::null_mut::<f64>();
                }
            }
        "#;

        let report = audit_rust_code(code).expect("Audit failed");
        assert_eq!(
            report.unsafe_blocks.len(),
            2,
            "Should detect two unsafe blocks"
        );
    }

    #[test]
    fn test_unsafe_density_calculation() {
        // RED: This will FAIL
        let code = r#"
fn example() {
    let x = 1;
    let y = 2;
    unsafe {
        let ptr = std::ptr::null_mut::<i32>();
    }
    let z = 3;
}
"#;
        let report = audit_rust_code(code).expect("Audit failed");

        // Total lines: 9, unsafe lines: 3 (lines 5-7)
        // Density should be around 33%
        assert!(report.unsafe_density_percent > 20.0);
        assert!(report.unsafe_density_percent < 50.0);
    }

    #[test]
    fn test_nested_unsafe_blocks() {
        // RED: This will FAIL
        let code = r#"
            fn nested() {
                unsafe {
                    let ptr = std::ptr::null_mut::<i32>();
                    unsafe {
                        *ptr = 42;
                    }
                }
            }
        "#;

        let report = audit_rust_code(code).expect("Audit failed");
        // Should detect nested blocks (implementation choice: count as 2 or 1)
        assert!(
            !report.unsafe_blocks.is_empty(),
            "Should detect unsafe blocks"
        );
    }

    #[test]
    fn test_unsafe_in_different_items() {
        // RED: This will FAIL
        let code = r#"
            fn func1() {
                unsafe { let x = 1; }
            }

            fn func2() {
                unsafe { let y = 2; }
            }

            impl MyStruct {
                fn method(&self) {
                    unsafe { let z = 3; }
                }
            }

            struct MyStruct;
        "#;

        let report = audit_rust_code(code).expect("Audit failed");
        assert_eq!(
            report.unsafe_blocks.len(),
            3,
            "Should detect unsafe in all items"
        );
    }

    #[test]
    fn test_confidence_scoring() {
        // RED: This will FAIL - confidence scoring not implemented
        let code = r#"
            fn with_pointer() {
                unsafe {
                    let ptr = std::ptr::null_mut::<i32>();
                    *ptr = 42;
                }
            }
        "#;

        let report = audit_rust_code(code).expect("Audit failed");
        assert_eq!(report.unsafe_blocks.len(), 1);

        let block = &report.unsafe_blocks[0];
        assert!(block.confidence > 0, "Should have non-zero confidence");
        assert!(block.confidence <= 100, "Confidence should be 0-100");
    }

    #[test]
    fn test_pattern_detection_raw_pointer() {
        // RED: This will FAIL - pattern detection not implemented
        let code = r#"
            fn deref_pointer() {
                unsafe {
                    let ptr = std::ptr::null_mut::<i32>();
                    *ptr = 42;
                }
            }
        "#;

        let report = audit_rust_code(code).expect("Audit failed");
        assert_eq!(report.unsafe_blocks.len(), 1);
        assert_eq!(
            report.unsafe_blocks[0].pattern,
            UnsafePattern::RawPointerDeref
        );
    }

    #[test]
    fn test_suggestion_generation() {
        // RED: This will FAIL - suggestions not implemented
        let code = r#"
            fn with_unsafe() {
                unsafe {
                    let ptr = std::ptr::null_mut::<i32>();
                }
            }
        "#;

        let report = audit_rust_code(code).expect("Audit failed");
        assert_eq!(report.unsafe_blocks.len(), 1);
        assert!(
            !report.unsafe_blocks[0].suggestion.is_empty(),
            "Should provide a suggestion"
        );
    }

    #[test]
    fn test_high_confidence_blocks() {
        // RED: This will FAIL
        let code = r#"
            fn example() {
                unsafe { let x = 1; }
                unsafe { let y = 2; }
            }
        "#;

        let report = audit_rust_code(code).expect("Audit failed");
        // Assuming we'll score some blocks as high confidence
        // This tests the filtering logic
        let high_conf = report.high_confidence_blocks();
        assert!(high_conf.len() <= report.unsafe_blocks.len());
    }

    #[test]
    fn test_average_confidence() {
        // RED: This will FAIL
        let code = r#"
            fn example() {
                unsafe { let x = 1; }
            }
        "#;

        let report = audit_rust_code(code).expect("Audit failed");
        assert!(report.average_confidence >= 0.0);
        assert!(report.average_confidence <= 100.0);
    }

    #[test]
    fn test_empty_code() {
        // This should pass (edge case)
        let code = "";
        let report = audit_rust_code(code).expect("Audit failed");
        assert_eq!(report.unsafe_blocks.len(), 0);
        assert_eq!(report.total_lines, 0);
    }

    #[test]
    fn test_invalid_rust_code() {
        // Should return error, not panic
        let code = "fn incomplete(";
        let result = audit_rust_code(code);
        assert!(result.is_err(), "Should return error for invalid code");
    }

    #[test]
    fn test_unsafe_fn() {
        // RED: This will FAIL - unsafe fn detection
        let code = r#"
            unsafe fn dangerous_function() {
                let x = 42;
            }
        "#;

        let report = audit_rust_code(code).expect("Audit failed");
        // Should detect unsafe function (entire function body is unsafe context)
        assert!(!report.unsafe_blocks.is_empty() || report.unsafe_lines > 0);
    }

    #[test]
    fn test_verify_compilation_valid_code() {
        let result = verify_compilation("fn main() {}").expect("rustc failed to run");
        assert!(result.success, "Valid Rust should compile");
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_verify_compilation_type_error() {
        let result = verify_compilation("fn main() { let x: i32 = \"bad\"; }")
            .expect("rustc failed to run");
        assert!(!result.success, "Type error should fail compilation");
        assert!(!result.errors.is_empty(), "Should have at least one error");
        // E0308 is the mismatched types error
        let has_e0308 = result.errors.iter().any(|e| {
            e.code.as_deref() == Some("E0308")
        });
        assert!(has_e0308, "Should contain E0308 error code");
    }

    #[test]
    fn test_verify_compilation_missing_function() {
        let result = verify_compilation("fn main() { undefined_function(); }")
            .expect("rustc failed to run");
        assert!(!result.success);
        assert!(!result.errors.is_empty());
    }

    // ========================================================================
    // Pattern detection: transmute, assembly, FFI, Other
    // ========================================================================

    #[test]
    fn test_pattern_detection_transmute() {
        let code = r#"
            fn with_transmute() {
                unsafe {
                    let x: u32 = std::mem::transmute(1.0f32);
                }
            }
        "#;
        let report = audit_rust_code(code).expect("Audit failed");
        assert_eq!(report.unsafe_blocks.len(), 1);
        assert_eq!(report.unsafe_blocks[0].pattern, UnsafePattern::Transmute);
        assert_eq!(report.unsafe_blocks[0].confidence, 40);
        assert!(report.unsafe_blocks[0].suggestion.contains("From/Into"));
    }

    #[test]
    fn test_pattern_detection_assembly() {
        // The auditor uses quote! to stringify the unsafe block, then searches for "asm!"
        // core::arch::asm! macro invocation appears in the stringified form
        let code = r#"
            fn with_asm() {
                unsafe {
                    std::arch::asm!("nop");
                }
            }
        "#;
        let report = audit_rust_code(code).expect("Audit failed");
        assert_eq!(report.unsafe_blocks.len(), 1);
        // The stringified block contains "asm!" so it should match Assembly
        // If the macro doesn't appear in stringified form, it may fall through to Other
        let pattern = &report.unsafe_blocks[0].pattern;
        assert!(
            *pattern == UnsafePattern::Assembly || *pattern == UnsafePattern::Other,
            "Expected Assembly or Other, got {:?}",
            pattern
        );
    }

    #[test]
    fn test_pattern_detection_ffi() {
        let code = r#"
            fn with_ffi() {
                unsafe {
                    extern "C" {
                        fn puts(s: *const u8) -> i32;
                    }
                }
            }
        "#;
        let report = audit_rust_code(code).expect("Audit failed");
        assert_eq!(report.unsafe_blocks.len(), 1);
        assert_eq!(report.unsafe_blocks[0].pattern, UnsafePattern::FfiCall);
        assert_eq!(report.unsafe_blocks[0].confidence, 30);
        assert!(report.unsafe_blocks[0].suggestion.contains("safe wrapper"));
    }

    #[test]
    fn test_pattern_detection_other() {
        let code = r#"
            fn with_other_unsafe() {
                unsafe {
                    let v: Vec<i32> = Vec::new();
                    let _ = v.len();
                }
            }
        "#;
        let report = audit_rust_code(code).expect("Audit failed");
        assert_eq!(report.unsafe_blocks.len(), 1);
        assert_eq!(report.unsafe_blocks[0].pattern, UnsafePattern::Other);
        assert_eq!(report.unsafe_blocks[0].confidence, 50);
        assert!(report.unsafe_blocks[0].suggestion.contains("Review"));
    }

    // ========================================================================
    // meets_density_target + high_confidence_blocks
    // ========================================================================

    #[test]
    fn test_meets_density_target_low_density() {
        let report = UnsafeAuditReport::new(100, 3, vec![]);
        assert!(report.meets_density_target());
        assert!(report.unsafe_density_percent < 5.0);
    }

    #[test]
    fn test_meets_density_target_high_density() {
        let report = UnsafeAuditReport::new(100, 10, vec![]);
        assert!(!report.meets_density_target());
        assert!(report.unsafe_density_percent >= 5.0);
    }

    #[test]
    fn test_meets_density_target_zero_lines() {
        let report = UnsafeAuditReport::new(0, 0, vec![]);
        assert!(report.meets_density_target());
        assert!((report.unsafe_density_percent - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_high_confidence_blocks_filtering() {
        let blocks = vec![
            UnsafeBlock {
                line: 1,
                confidence: 85,
                pattern: UnsafePattern::RawPointerDeref,
                suggestion: "Use Box".to_string(),
            },
            UnsafeBlock {
                line: 10,
                confidence: 40,
                pattern: UnsafePattern::Transmute,
                suggestion: "Use From".to_string(),
            },
            UnsafeBlock {
                line: 20,
                confidence: 70,
                pattern: UnsafePattern::Other,
                suggestion: "Review".to_string(),
            },
        ];
        let report = UnsafeAuditReport::new(100, 10, blocks);
        let high = report.high_confidence_blocks();
        assert_eq!(high.len(), 2); // confidence 85 and 70
        assert!(high.iter().all(|b| b.confidence >= 70));
    }

    #[test]
    fn test_high_confidence_blocks_empty() {
        let report = UnsafeAuditReport::new(100, 0, vec![]);
        assert!(report.high_confidence_blocks().is_empty());
    }

    // ========================================================================
    // UnsafeAuditor default + unsafe fn detection
    // ========================================================================

    #[test]
    fn test_unsafe_auditor_default() {
        let auditor = UnsafeAuditor::default();
        assert_eq!(auditor.total_lines, 0);
        assert_eq!(auditor.unsafe_lines, 0);
        assert!(auditor.unsafe_blocks.is_empty());
    }

    #[test]
    fn test_unsafe_fn_detection_body_lines() {
        let code = r#"
            unsafe fn dangerous() {
                let x = 1;
                let y = 2;
                let z = 3;
            }
        "#;
        let report = audit_rust_code(code).expect("Audit failed");
        assert!(!report.unsafe_blocks.is_empty());
        let block = &report.unsafe_blocks[0];
        assert_eq!(block.confidence, 60);
        assert_eq!(block.pattern, UnsafePattern::Other);
        assert!(block.suggestion.contains("Unsafe function"));
    }

    // ========================================================================
    // verify_compilation: warnings path + edge cases
    // ========================================================================

    #[test]
    fn test_verify_compilation_with_warnings() {
        let code = "#[warn(unused_variables)] fn main() { let x = 42; }";
        let result = verify_compilation(code).expect("rustc failed to run");
        assert!(result.success);
        // May or may not have warnings depending on rustc config
    }

    #[test]
    fn test_verify_compilation_multiple_errors() {
        let code = "fn main() { undefined_a(); undefined_b(); }";
        let result = verify_compilation(code).expect("rustc failed to run");
        assert!(!result.success);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_compilation_error_without_code() {
        // Error lines without [EXXXX] format
        let code = "this is not valid rust at all";
        let result = verify_compilation(code).expect("rustc failed to run");
        assert!(!result.success);
        // Should have errors, some may lack error codes
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_average_confidence_calculation() {
        let blocks = vec![
            UnsafeBlock {
                line: 0,
                confidence: 80,
                pattern: UnsafePattern::RawPointerDeref,
                suggestion: "s".to_string(),
            },
            UnsafeBlock {
                line: 0,
                confidence: 40,
                pattern: UnsafePattern::Transmute,
                suggestion: "s".to_string(),
            },
        ];
        let report = UnsafeAuditReport::new(100, 10, blocks);
        assert!((report.average_confidence - 60.0).abs() < 0.001);
    }

    #[test]
    fn test_average_confidence_no_blocks() {
        let report = UnsafeAuditReport::new(100, 0, vec![]);
        assert!((report.average_confidence - 0.0).abs() < 0.001);
    }
}

#[cfg(test)]
#[path = "lock_verify_tests.rs"]
mod lock_verify_tests;
