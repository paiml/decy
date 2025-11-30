//! Verification and iteration framework for LLM-generated code (DECY-100).
//!
//! Verifies that generated Rust code compiles, passes tests, and
//! iterates on failures with error feedback.

use crate::llm_codegen::{GeneratedCode, LlmError};
use serde::{Deserialize, Serialize};

/// Result of code verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether the code compiled successfully
    pub compiles: bool,
    /// Whether all tests passed
    pub tests_pass: bool,
    /// Compilation errors (if any)
    pub compile_errors: Vec<String>,
    /// Test failures (if any)
    pub test_failures: Vec<String>,
    /// Number of clippy warnings
    pub clippy_warnings: usize,
    /// Overall success
    pub success: bool,
}

impl VerificationResult {
    /// Create a successful result.
    pub fn success() -> Self {
        Self {
            compiles: true,
            tests_pass: true,
            compile_errors: Vec::new(),
            test_failures: Vec::new(),
            clippy_warnings: 0,
            success: true,
        }
    }

    /// Create a compilation failure result.
    pub fn compile_failure(errors: Vec<String>) -> Self {
        Self {
            compiles: false,
            tests_pass: false,
            compile_errors: errors,
            test_failures: Vec::new(),
            clippy_warnings: 0,
            success: false,
        }
    }

    /// Create a test failure result.
    pub fn test_failure(failures: Vec<String>) -> Self {
        Self {
            compiles: true,
            tests_pass: false,
            compile_errors: Vec::new(),
            test_failures: failures,
            clippy_warnings: 0,
            success: false,
        }
    }
}

/// Iteration context for retry with error feedback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationContext {
    /// Current iteration number (1-based)
    pub iteration: usize,
    /// Maximum iterations allowed
    pub max_iterations: usize,
    /// Previous generated code
    pub previous_code: Option<String>,
    /// Previous errors
    pub previous_errors: Vec<String>,
    /// Accumulated feedback
    pub feedback: Vec<String>,
}

impl IterationContext {
    /// Create a new iteration context.
    pub fn new(max_iterations: usize) -> Self {
        Self {
            iteration: 1,
            max_iterations,
            previous_code: None,
            previous_errors: Vec::new(),
            feedback: Vec::new(),
        }
    }

    /// Check if more iterations are allowed.
    pub fn can_retry(&self) -> bool {
        self.iteration <= self.max_iterations
    }

    /// Record a failed iteration.
    pub fn record_failure(&mut self, code: &str, errors: Vec<String>) {
        self.previous_code = Some(code.to_string());
        self.previous_errors = errors.clone();

        // Add to feedback
        for error in &errors {
            self.feedback
                .push(format!("Iteration {}: {}", self.iteration, error));
        }

        self.iteration += 1;
    }

    /// Get formatted feedback for next iteration.
    pub fn get_feedback(&self) -> String {
        let mut feedback = String::new();

        feedback.push_str("## Previous Errors\n\n");

        for error in &self.previous_errors {
            feedback.push_str("- ");
            feedback.push_str(error);
            feedback.push('\n');
        }

        if let Some(ref code) = self.previous_code {
            feedback.push_str("\n## Previous Code\n```rust\n");
            feedback.push_str(code);
            feedback.push_str("\n```\n");
        }

        feedback.push_str("\n## Instructions\n");
        feedback.push_str("Please fix the errors above and generate corrected Rust code.\n");

        feedback
    }
}

/// Code verifier that compiles and tests generated Rust.
#[derive(Debug)]
pub struct CodeVerifier {
    /// Temporary directory for compilation (not used in stub)
    _temp_dir: Option<std::path::PathBuf>,
}

impl CodeVerifier {
    /// Create a new code verifier.
    pub fn new() -> Self {
        Self { _temp_dir: None }
    }

    /// Verify generated code by compiling it.
    ///
    /// Note: This is a stub for research purposes. Full implementation
    /// would create a temporary project and run cargo build.
    pub fn verify(&self, code: &GeneratedCode) -> Result<VerificationResult, LlmError> {
        // Basic validation - check if code looks valid
        if code.code.trim().is_empty() {
            return Ok(VerificationResult::compile_failure(vec![
                "Empty code".to_string()
            ]));
        }

        // Check for balanced braces (basic syntax check)
        let open = code.code.matches('{').count();
        let close = code.code.matches('}').count();

        if open != close {
            return Ok(VerificationResult::compile_failure(vec![format!(
                "Unbalanced braces: {} open, {} close",
                open, close
            )]));
        }

        // For research purposes, assume valid-looking code compiles
        Ok(VerificationResult::success())
    }

    /// Try to compile Rust code.
    ///
    /// Stub implementation - full version would use cargo.
    pub fn compile(&self, code: &str) -> Result<(), Vec<String>> {
        if code.trim().is_empty() {
            return Err(vec!["Empty code".to_string()]);
        }

        // Basic syntax checks
        let open_braces = code.matches('{').count();
        let close_braces = code.matches('}').count();

        if open_braces != close_braces {
            return Err(vec![format!(
                "Unbalanced braces: {} open, {} close",
                open_braces, close_braces
            )]);
        }

        Ok(())
    }

    /// Run clippy on the code.
    ///
    /// Stub implementation - returns 0 warnings for valid code.
    pub fn lint(&self, code: &str) -> Result<usize, LlmError> {
        // Basic check - count potential issues
        let mut warnings = 0;

        // Check for common issues
        if code.contains("unwrap()") {
            warnings += 1;
        }
        if code.contains("expect(") {
            warnings += 1;
        }
        if code.contains("panic!") {
            warnings += 1;
        }

        Ok(warnings)
    }

    /// Run tests if present.
    ///
    /// Stub implementation.
    pub fn run_tests(&self, code: &str) -> Result<(), Vec<String>> {
        // Check if there are test functions
        if code.contains("#[test]") {
            // Assume tests pass for valid code in stub
            Ok(())
        } else {
            // No tests to run
            Ok(())
        }
    }

    /// Create a temporary project for compilation.
    ///
    /// Stub implementation.
    fn _create_temp_project(&self, _code: &str) -> Result<std::path::PathBuf, LlmError> {
        Err(LlmError::ApiError(
            "Temporary project creation not implemented".to_string(),
        ))
    }
}

impl Default for CodeVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Verification-iteration loop runner.
#[derive(Debug)]
pub struct VerificationLoop {
    /// Maximum iterations
    max_iterations: usize,
}

impl VerificationLoop {
    /// Create a new verification loop.
    pub fn new(max_iterations: usize) -> Self {
        Self { max_iterations }
    }

    /// Get max iterations.
    pub fn max_iterations(&self) -> usize {
        self.max_iterations
    }

    /// Check if a result indicates success.
    pub fn is_success(&self, result: &VerificationResult) -> bool {
        result.success && result.compiles && result.tests_pass
    }

    /// Format errors for feedback.
    pub fn format_feedback(&self, result: &VerificationResult) -> String {
        let mut feedback = String::new();

        if !result.compile_errors.is_empty() {
            feedback.push_str("## Compilation Errors\n\n");
            for error in &result.compile_errors {
                feedback.push_str("- ");
                feedback.push_str(error);
                feedback.push('\n');
            }
        }

        if !result.test_failures.is_empty() {
            feedback.push_str("\n## Test Failures\n\n");
            for failure in &result.test_failures {
                feedback.push_str("- ");
                feedback.push_str(failure);
                feedback.push('\n');
            }
        }

        if result.clippy_warnings > 0 {
            feedback.push_str(&format!(
                "\n## Clippy Warnings: {}\n",
                result.clippy_warnings
            ));
        }

        feedback
    }
}

impl Default for VerificationLoop {
    fn default() -> Self {
        Self::new(3)
    }
}
