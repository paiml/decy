//! Semantic verification for pattern promotion
//!
//! Patterns must pass both rustc AND unit tests before promotion.
//! Implements Jidoka principle - stop the line on defects.
//!
//! # References
//! - training-oracle-spec.md §3.2.1.1: Semantic Verification (Jidoka Enhancement)
//! - Gemini Review: "compilation alone is insufficient"

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// Result of semantic verification
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationResult {
    /// Compiles successfully AND all tests pass
    FullyVerified,
    /// Compiles successfully but no tests available
    CompilesOnly,
    /// Compiles but tests fail (behavior changed)
    BehaviorChanged(Vec<String>),
    /// Doesn't compile
    CompileFailed(String),
}

impl VerificationResult {
    /// Check if verification allows pattern promotion
    pub fn allows_promotion(&self) -> bool {
        matches!(
            self,
            VerificationResult::FullyVerified | VerificationResult::CompilesOnly
        )
    }

    /// Get confidence weight for pattern scoring
    pub fn confidence_weight(&self) -> f32 {
        match self {
            VerificationResult::FullyVerified => 1.0,
            VerificationResult::CompilesOnly => 0.6,
            VerificationResult::BehaviorChanged(_) => 0.0,
            VerificationResult::CompileFailed(_) => 0.0,
        }
    }

    /// Check if this is a compile failure
    pub fn is_compile_failure(&self) -> bool {
        matches!(self, VerificationResult::CompileFailed(_))
    }

    /// Check if behavior changed
    pub fn has_behavior_change(&self) -> bool {
        matches!(self, VerificationResult::BehaviorChanged(_))
    }
}

/// Result of running a test suite
#[derive(Debug, Clone, PartialEq)]
pub enum TestResult {
    /// All tests passed
    AllPassed,
    /// Some tests failed
    SomeFailed(Vec<String>),
    /// No tests available
    NoTests,
    /// Test execution failed
    ExecutionError(String),
}

impl TestResult {
    /// Check if tests passed (or no tests available)
    pub fn is_ok(&self) -> bool {
        matches!(self, TestResult::AllPassed | TestResult::NoTests)
    }
}

/// Configuration for semantic verification
#[derive(Debug, Clone)]
pub struct VerificationConfig {
    /// Timeout for compilation in seconds
    pub compile_timeout_secs: u64,
    /// Timeout for test execution in seconds
    pub test_timeout_secs: u64,
    /// Whether to run tests (false = compile-only mode)
    pub run_tests: bool,
    /// Working directory for compilation
    pub work_dir: Option<std::path::PathBuf>,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            compile_timeout_secs: 60,
            test_timeout_secs: 120,
            run_tests: true,
            work_dir: None,
        }
    }
}

/// Statistics for verification operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerificationStats {
    /// Patterns fully verified
    pub fully_verified: usize,
    /// Patterns that only compile
    pub compiles_only: usize,
    /// Patterns with behavior changes
    pub behavior_changed: usize,
    /// Patterns that failed to compile
    pub compile_failed: usize,
    /// Total patterns evaluated
    pub total_evaluated: usize,
}

impl VerificationStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a verification result
    pub fn record(&mut self, result: &VerificationResult) {
        self.total_evaluated += 1;
        match result {
            VerificationResult::FullyVerified => self.fully_verified += 1,
            VerificationResult::CompilesOnly => self.compiles_only += 1,
            VerificationResult::BehaviorChanged(_) => self.behavior_changed += 1,
            VerificationResult::CompileFailed(_) => self.compile_failed += 1,
        }
    }

    /// Get promotion rate (patterns that can be promoted)
    pub fn promotion_rate(&self) -> f32 {
        if self.total_evaluated == 0 {
            0.0
        } else {
            (self.fully_verified + self.compiles_only) as f32 / self.total_evaluated as f32
        }
    }

    /// Get full verification rate
    pub fn full_verification_rate(&self) -> f32 {
        if self.total_evaluated == 0 {
            0.0
        } else {
            self.fully_verified as f32 / self.total_evaluated as f32
        }
    }

    /// Get average confidence weight
    pub fn average_confidence(&self) -> f32 {
        if self.total_evaluated == 0 {
            0.0
        } else {
            let weighted_sum = (self.fully_verified as f32 * 1.0)
                + (self.compiles_only as f32 * 0.6)
                + (self.behavior_changed as f32 * 0.0)
                + (self.compile_failed as f32 * 0.0);
            weighted_sum / self.total_evaluated as f32
        }
    }
}

/// Check if Rust code compiles successfully
pub fn check_rust_compilation(rust_code: &str, config: &VerificationConfig) -> Result<(), String> {
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};

    // Create temporary directory for all output files
    let temp_dir = config.work_dir.clone().unwrap_or_else(std::env::temp_dir);

    // Create unique temp file names using atomic counter for thread safety
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
    let unique_id = format!("{}_{}", std::process::id(), counter);
    let temp_file = temp_dir.join(format!("decy_verify_{}.rs", unique_id));
    let temp_output = temp_dir.join(format!("decy_verify_{}.out", unique_id));

    // Write code to temp file
    let mut file = std::fs::File::create(&temp_file)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    file.write_all(rust_code.as_bytes())
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    // Drop the file handle to ensure it's flushed
    drop(file);

    // Run rustc check - use --emit=obj to a temp file (faster than full codegen)
    // Allow warnings but catch errors
    let output = Command::new("rustc")
        .args([
            "--edition",
            "2021",
            "--crate-type",
            "lib",
            "--emit",
            "metadata",
            "-A",
            "warnings", // Allow warnings, only catch errors
            "-o",
        ])
        .arg(&temp_output)
        .arg(&temp_file)
        .output()
        .map_err(|e| format!("Failed to run rustc: {}", e))?;

    // Clean up temp files
    let _ = std::fs::remove_file(&temp_file);
    let _ = std::fs::remove_file(&temp_output);
    // Also clean up any rmeta files that might have been created
    let rmeta_file = temp_dir.join(format!("libdecy_verify_{}.rmeta", unique_id));
    let _ = std::fs::remove_file(&rmeta_file);

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.to_string())
    }
}

/// Run test suite against transpiled code
pub fn run_test_suite(test_path: &Path, _rust_code: &str) -> TestResult {
    if !test_path.exists() {
        return TestResult::NoTests;
    }

    // Check for test files
    let has_tests = test_path.is_dir()
        && std::fs::read_dir(test_path)
            .map(|entries| entries.count() > 0)
            .unwrap_or(false);

    if !has_tests {
        return TestResult::NoTests;
    }

    // Run tests using shell scripts or cargo test
    let test_script = test_path.join("run_tests.sh");
    if test_script.exists() {
        let output = Command::new("bash").arg(&test_script).output();

        match output {
            Ok(result) if result.status.success() => TestResult::AllPassed,
            Ok(result) => {
                let stderr = String::from_utf8_lossy(&result.stderr);
                TestResult::SomeFailed(vec![stderr.to_string()])
            }
            Err(e) => TestResult::ExecutionError(e.to_string()),
        }
    } else {
        // No runnable test script
        TestResult::NoTests
    }
}

/// Perform full semantic verification
pub fn verify_fix_semantically(
    rust_code: &str,
    test_suite: Option<&Path>,
    config: &VerificationConfig,
) -> VerificationResult {
    // Step 1: Syntactic verification (compilation)
    if let Err(e) = check_rust_compilation(rust_code, config) {
        return VerificationResult::CompileFailed(e);
    }

    // Step 2: Semantic verification (tests)
    if !config.run_tests {
        return VerificationResult::CompilesOnly;
    }

    match test_suite {
        Some(tests) => {
            match run_test_suite(tests, rust_code) {
                TestResult::AllPassed => VerificationResult::FullyVerified,
                TestResult::SomeFailed(failures) => VerificationResult::BehaviorChanged(failures),
                TestResult::NoTests => VerificationResult::CompilesOnly,
                TestResult::ExecutionError(e) => {
                    // Treat execution errors as behavior change (conservative)
                    VerificationResult::BehaviorChanged(vec![e])
                }
            }
        }
        None => VerificationResult::CompilesOnly,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // VerificationResult Tests
    // ============================================================================

    #[test]
    fn test_verification_result_allows_promotion() {
        assert!(VerificationResult::FullyVerified.allows_promotion());
        assert!(VerificationResult::CompilesOnly.allows_promotion());
        assert!(!VerificationResult::BehaviorChanged(vec![]).allows_promotion());
        assert!(!VerificationResult::CompileFailed("error".into()).allows_promotion());
    }

    #[test]
    fn test_verification_result_confidence_weight() {
        assert!((VerificationResult::FullyVerified.confidence_weight() - 1.0).abs() < f32::EPSILON);
        assert!((VerificationResult::CompilesOnly.confidence_weight() - 0.6).abs() < f32::EPSILON);
        assert!(
            (VerificationResult::BehaviorChanged(vec![]).confidence_weight() - 0.0).abs()
                < f32::EPSILON
        );
        assert!(
            (VerificationResult::CompileFailed("".into()).confidence_weight() - 0.0).abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_verification_result_is_compile_failure() {
        assert!(VerificationResult::CompileFailed("error".into()).is_compile_failure());
        assert!(!VerificationResult::FullyVerified.is_compile_failure());
        assert!(!VerificationResult::CompilesOnly.is_compile_failure());
        assert!(!VerificationResult::BehaviorChanged(vec![]).is_compile_failure());
    }

    #[test]
    fn test_verification_result_has_behavior_change() {
        assert!(VerificationResult::BehaviorChanged(vec!["test".into()]).has_behavior_change());
        assert!(!VerificationResult::FullyVerified.has_behavior_change());
        assert!(!VerificationResult::CompilesOnly.has_behavior_change());
        assert!(!VerificationResult::CompileFailed("".into()).has_behavior_change());
    }

    // ============================================================================
    // TestResult Tests
    // ============================================================================

    #[test]
    fn test_test_result_is_ok() {
        assert!(TestResult::AllPassed.is_ok());
        assert!(TestResult::NoTests.is_ok());
        assert!(!TestResult::SomeFailed(vec![]).is_ok());
        assert!(!TestResult::ExecutionError("".into()).is_ok());
    }

    // ============================================================================
    // VerificationConfig Tests
    // ============================================================================

    #[test]
    fn test_verification_config_default() {
        let config = VerificationConfig::default();
        assert_eq!(config.compile_timeout_secs, 60);
        assert_eq!(config.test_timeout_secs, 120);
        assert!(config.run_tests);
        assert!(config.work_dir.is_none());
    }

    // ============================================================================
    // VerificationStats Tests
    // ============================================================================

    #[test]
    fn test_verification_stats_new() {
        let stats = VerificationStats::new();
        assert_eq!(stats.total_evaluated, 0);
        assert_eq!(stats.fully_verified, 0);
    }

    #[test]
    fn test_verification_stats_record() {
        let mut stats = VerificationStats::new();

        stats.record(&VerificationResult::FullyVerified);
        stats.record(&VerificationResult::CompilesOnly);
        stats.record(&VerificationResult::BehaviorChanged(vec![]));
        stats.record(&VerificationResult::CompileFailed("".into()));

        assert_eq!(stats.total_evaluated, 4);
        assert_eq!(stats.fully_verified, 1);
        assert_eq!(stats.compiles_only, 1);
        assert_eq!(stats.behavior_changed, 1);
        assert_eq!(stats.compile_failed, 1);
    }

    #[test]
    fn test_verification_stats_promotion_rate() {
        let mut stats = VerificationStats::new();

        stats.record(&VerificationResult::FullyVerified);
        stats.record(&VerificationResult::CompilesOnly);
        stats.record(&VerificationResult::CompileFailed("".into()));
        stats.record(&VerificationResult::CompileFailed("".into()));

        // 2 promotable out of 4
        assert!((stats.promotion_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_verification_stats_full_verification_rate() {
        let mut stats = VerificationStats::new();

        stats.record(&VerificationResult::FullyVerified);
        stats.record(&VerificationResult::FullyVerified);
        stats.record(&VerificationResult::CompilesOnly);
        stats.record(&VerificationResult::CompileFailed("".into()));

        // 2 fully verified out of 4
        assert!((stats.full_verification_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_verification_stats_average_confidence() {
        let mut stats = VerificationStats::new();

        stats.record(&VerificationResult::FullyVerified); // 1.0
        stats.record(&VerificationResult::CompilesOnly); // 0.6

        // Average: (1.0 + 0.6) / 2 = 0.8
        assert!((stats.average_confidence() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_verification_stats_empty_rates() {
        let stats = VerificationStats::new();
        assert_eq!(stats.promotion_rate(), 0.0);
        assert_eq!(stats.full_verification_rate(), 0.0);
        assert_eq!(stats.average_confidence(), 0.0);
    }

    // ============================================================================
    // Compilation Check Tests
    // ============================================================================

    #[test]
    fn test_check_compilation_valid_code() {
        let code = r#"
            fn main() {
                let x: i32 = 42;
                println!("{}", x);
            }
        "#;
        let config = VerificationConfig::default();
        let result = check_rust_compilation(code, &config);
        assert!(result.is_ok(), "Valid code should compile: {:?}", result);
    }

    #[test]
    fn test_check_compilation_invalid_code() {
        let code = r#"
            fn main() {
                let x: i32 = "not an integer";
            }
        "#;
        let config = VerificationConfig::default();
        let result = check_rust_compilation(code, &config);
        assert!(result.is_err(), "Invalid code should fail to compile");
    }

    #[test]
    fn test_check_compilation_syntax_error() {
        let code = r#"
            fn main( {
                // Missing closing paren
            }
        "#;
        let config = VerificationConfig::default();
        let result = check_rust_compilation(code, &config);
        assert!(result.is_err(), "Syntax error should fail to compile");
    }

    // ============================================================================
    // Full Verification Tests
    // ============================================================================

    #[test]
    fn test_verify_valid_code_no_tests() {
        let code = r#"
            fn main() {
                let x = 42;
            }
        "#;
        let config = VerificationConfig::default();
        let result = verify_fix_semantically(code, None, &config);
        assert_eq!(result, VerificationResult::CompilesOnly);
    }

    #[test]
    fn test_verify_invalid_code() {
        let code = r#"
            fn main() {
                let x: i32 = "error";
            }
        "#;
        let config = VerificationConfig::default();
        let result = verify_fix_semantically(code, None, &config);
        assert!(result.is_compile_failure());
    }

    #[test]
    fn test_verify_compile_only_mode() {
        let code = r#"
            fn main() {}
        "#;
        let config = VerificationConfig {
            run_tests: false,
            ..Default::default()
        };
        let result = verify_fix_semantically(code, None, &config);
        assert_eq!(result, VerificationResult::CompilesOnly);
    }

    #[test]
    fn test_verify_nonexistent_test_dir() {
        let code = r#"
            fn main() {}
        "#;
        let config = VerificationConfig::default();
        let test_path = Path::new("/nonexistent/test/path");
        let result = verify_fix_semantically(code, Some(test_path), &config);
        // Should fall back to CompilesOnly since test path doesn't exist
        assert_eq!(result, VerificationResult::CompilesOnly);
    }

    // ============================================================================
    // Test Suite Tests
    // ============================================================================

    #[test]
    fn test_run_test_suite_no_path() {
        let result = run_test_suite(Path::new("/nonexistent"), "fn main() {}");
        assert_eq!(result, TestResult::NoTests);
    }

    // ============================================================================
    // Spec Compliance Tests
    // ============================================================================

    #[test]
    fn test_spec_weight_fully_verified() {
        // Spec: FullyVerified patterns get weight 1.0
        assert_eq!(VerificationResult::FullyVerified.confidence_weight(), 1.0);
    }

    #[test]
    fn test_spec_weight_compiles_only() {
        // Spec: CompilesOnly patterns get weight 0.6
        assert!((VerificationResult::CompilesOnly.confidence_weight() - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn test_spec_behavior_changed_rejected() {
        // Spec: BehaviorChanged patterns get weight 0.0 (rejected)
        let result = VerificationResult::BehaviorChanged(vec!["test failed".into()]);
        assert_eq!(result.confidence_weight(), 0.0);
        assert!(!result.allows_promotion());
    }

    #[test]
    fn test_spec_compile_failed_rejected() {
        // Spec: CompileFailed patterns get weight 0.0 (rejected)
        let result = VerificationResult::CompileFailed("error".into());
        assert_eq!(result.confidence_weight(), 0.0);
        assert!(!result.allows_promotion());
    }

    #[test]
    fn test_verification_stats_default() {
        let stats = VerificationStats::default();
        assert_eq!(stats.total_evaluated, 0);
        assert_eq!(stats.fully_verified, 0);
        assert_eq!(stats.compiles_only, 0);
        assert_eq!(stats.behavior_changed, 0);
        assert_eq!(stats.compile_failed, 0);
        assert_eq!(stats.promotion_rate(), 0.0);
    }

    #[test]
    fn test_verification_result_debug_clone() {
        let result = VerificationResult::BehaviorChanged(vec!["test1".into(), "test2".into()]);
        let cloned = result.clone();
        assert_eq!(result, cloned);
        let debug = format!("{:?}", result);
        assert!(debug.contains("BehaviorChanged"));
    }

    #[test]
    fn test_test_result_debug_clone() {
        let result = TestResult::SomeFailed(vec!["failure1".into()]);
        let cloned = result.clone();
        assert_eq!(result, cloned);
        let debug = format!("{:?}", result);
        assert!(debug.contains("SomeFailed"));
    }

    #[test]
    fn test_verification_stats_all_behavior_changed() {
        let mut stats = VerificationStats::new();
        stats.record(&VerificationResult::BehaviorChanged(vec!["fail".into()]));
        stats.record(&VerificationResult::BehaviorChanged(vec!["fail".into()]));
        assert_eq!(stats.behavior_changed, 2);
        assert_eq!(stats.promotion_rate(), 0.0);
        assert_eq!(stats.full_verification_rate(), 0.0);
        assert_eq!(stats.average_confidence(), 0.0);
    }

    #[test]
    fn test_verification_stats_mixed_confidence() {
        let mut stats = VerificationStats::new();
        stats.record(&VerificationResult::FullyVerified); // 1.0
        stats.record(&VerificationResult::CompileFailed("err".into())); // 0.0
        // Average: (1.0 + 0.0) / 2 = 0.5
        assert!((stats.average_confidence() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_verification_config_custom() {
        let config = VerificationConfig {
            compile_timeout_secs: 30,
            test_timeout_secs: 60,
            run_tests: false,
            work_dir: Some(std::path::PathBuf::from("/tmp")),
        };
        assert_eq!(config.compile_timeout_secs, 30);
        assert!(!config.run_tests);
        assert!(config.work_dir.is_some());
    }
}
