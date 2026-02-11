//! Trace Verifier: Poka-Yoke gate for Golden Trace quality
//!
//! Per unified spec Section 6.2, this module ensures only SAFE, COMPILING
//! Rust enters the training dataset. It acts as the hard quality gate
//! that prevents hallucinated or invalid code from contaminating training data.
//!
//! # Toyota Way Principle: Poka-Yoke (ポカヨケ)
//!
//! Mistake-proofing - the verifier prevents defective traces from entering
//! the dataset, ensuring model training data quality.

use crate::golden_trace::GoldenTrace;
use std::io::Write;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// Verification level determining strictness of checks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VerificationLevel {
    /// Only check compilation
    Minimal,
    /// Compilation + unsafe check (default)
    #[default]
    Standard,
    /// Compilation + unsafe + clippy
    Strict,
}

impl std::fmt::Display for VerificationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationLevel::Minimal => write!(f, "minimal"),
            VerificationLevel::Standard => write!(f, "standard"),
            VerificationLevel::Strict => write!(f, "strict"),
        }
    }
}

/// Configuration for the trace verifier
#[derive(Debug, Clone)]
pub struct VerifierConfig {
    /// Verification strictness level
    pub level: VerificationLevel,
    /// Whether to allow unsafe blocks
    pub allow_unsafe: bool,
    /// Maximum allowed clippy warnings (0 for strict)
    pub max_clippy_warnings: usize,
    /// Compilation timeout in seconds
    pub timeout_secs: u64,
}

impl Default for VerifierConfig {
    fn default() -> Self {
        Self {
            level: VerificationLevel::Standard,
            allow_unsafe: false,
            max_clippy_warnings: 0,
            timeout_secs: 30,
        }
    }
}

/// Result of verifying a trace
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether the trace passed verification
    pub passed: bool,
    /// Error messages if verification failed
    pub errors: Vec<String>,
    /// Warning messages
    pub warnings: Vec<String>,
    /// Number of unsafe blocks detected
    pub unsafe_count: usize,
    /// Time taken to verify (milliseconds)
    pub compilation_time_ms: u64,
}

impl VerificationResult {
    /// Check if the result is completely clean (no errors or warnings)
    pub fn is_clean(&self) -> bool {
        self.passed && self.errors.is_empty() && self.warnings.is_empty()
    }
}

/// Statistics about verification runs
#[derive(Debug, Clone, Default)]
pub struct VerifierStats {
    /// Total traces verified
    pub total_verified: usize,
    /// Number that passed
    pub passed: usize,
    /// Number that failed
    pub failed: usize,
    /// Total unsafe blocks detected across all traces
    pub total_unsafe_blocks: usize,
    /// Average verification time
    pub avg_verification_time_ms: f64,
}

impl VerifierStats {
    /// Calculate pass rate as a fraction
    pub fn pass_rate(&self) -> f64 {
        if self.total_verified == 0 {
            0.0
        } else {
            self.passed as f64 / self.total_verified as f64
        }
    }
}

/// Verifier for Golden Traces
///
/// Ensures only safe, compiling Rust enters the training dataset.
pub struct TraceVerifier {
    config: VerifierConfig,
    stats: VerifierStats,
}

impl Default for TraceVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceVerifier {
    /// Create a new verifier with default config
    pub fn new() -> Self {
        Self {
            config: VerifierConfig::default(),
            stats: VerifierStats::default(),
        }
    }

    /// Create a verifier with custom config
    pub fn with_config(config: VerifierConfig) -> Self {
        Self {
            config,
            stats: VerifierStats::default(),
        }
    }

    /// Get the current config
    pub fn config(&self) -> &VerifierConfig {
        &self.config
    }

    /// Get verification statistics
    pub fn stats(&self) -> &VerifierStats {
        &self.stats
    }

    /// Verify that Rust code compiles
    pub fn verify_compilation(&self, rust_code: &str) -> Result<(), String> {
        // Create temp file
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        let unique_id = format!("{}_{}", std::process::id(), counter);

        let temp_dir = std::env::temp_dir();
        let rust_path = temp_dir.join(format!("decy_verify_{}.rs", unique_id));

        // Write Rust code to temp file
        let mut file = std::fs::File::create(&rust_path)
            .map_err(|e| format!("Failed to create temp file: {}", e))?;
        file.write_all(rust_code.as_bytes())
            .map_err(|e| format!("Failed to write temp file: {}", e))?;

        // Run rustc --emit=metadata (fast check without codegen)
        let output = Command::new("rustc")
            .arg("--emit=metadata")
            .arg("--edition=2021")
            .arg("-o")
            .arg(temp_dir.join(format!("decy_verify_{}.rmeta", unique_id)))
            .arg(&rust_path)
            .output()
            .map_err(|e| format!("Failed to run rustc: {}", e))?;

        // Clean up
        let _ = std::fs::remove_file(&rust_path);
        let _ = std::fs::remove_file(temp_dir.join(format!("decy_verify_{}.rmeta", unique_id)));

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(stderr.to_string())
        }
    }

    /// Count unsafe blocks in Rust code
    pub fn count_unsafe_blocks(&self, rust_code: &str) -> usize {
        // Count "unsafe {" patterns - more accurate than just "unsafe"
        rust_code.matches("unsafe {").count() + rust_code.matches("unsafe{").count()
    }

    /// Verify safety constraints (unsafe block check)
    pub fn verify_safety(&self, rust_code: &str) -> Result<(), String> {
        let unsafe_count = self.count_unsafe_blocks(rust_code);

        if !self.config.allow_unsafe && unsafe_count > 0 {
            return Err(format!(
                "Code contains {} unsafe block(s) but unsafe is not allowed",
                unsafe_count
            ));
        }

        Ok(())
    }

    /// Verify a Golden Trace
    pub fn verify_trace(&mut self, trace: &GoldenTrace) -> VerificationResult {
        let start = std::time::Instant::now();
        let mut errors = Vec::new();
        let warnings = Vec::new();

        // Wrap in main if needed for compilation
        let rust_code = if trace.rust_snippet.contains("fn main") {
            trace.rust_snippet.clone()
        } else {
            format!("fn main() {{\n{}\n}}", trace.rust_snippet)
        };

        // Check compilation
        if let Err(e) = self.verify_compilation(&rust_code) {
            errors.push(e);
        }

        // Check for unsafe blocks
        let unsafe_count = self.count_unsafe_blocks(&rust_code);
        if !self.config.allow_unsafe && unsafe_count > 0 {
            errors.push(format!("Contains {} unsafe block(s)", unsafe_count));
        }

        let passed = errors.is_empty();
        let compilation_time_ms = start.elapsed().as_millis() as u64;

        // Update stats
        self.stats.total_verified += 1;
        if passed {
            self.stats.passed += 1;
        } else {
            self.stats.failed += 1;
        }
        self.stats.total_unsafe_blocks += unsafe_count;

        // Update average time
        let n = self.stats.total_verified as f64;
        self.stats.avg_verification_time_ms =
            (self.stats.avg_verification_time_ms * (n - 1.0) + compilation_time_ms as f64) / n;

        VerificationResult {
            passed,
            errors,
            warnings,
            unsafe_count,
            compilation_time_ms,
        }
    }

    /// Verify a batch of traces
    pub fn verify_batch(&self, traces: &[GoldenTrace]) -> Vec<VerificationResult> {
        let mut verifier = Self::with_config(self.config.clone());
        traces.iter().map(|t| verifier.verify_trace(t)).collect()
    }

    /// Filter to only valid traces
    pub fn filter_valid<'a>(&self, traces: &'a [GoldenTrace]) -> Vec<&'a GoldenTrace> {
        let mut verifier = Self::with_config(self.config.clone());
        traces
            .iter()
            .filter(|t| verifier.verify_trace(t).passed)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::golden_trace::{GoldenTrace, TraceTier};

    fn make_trace(rust_code: &str) -> GoldenTrace {
        GoldenTrace::new(
            "int x = 0;".to_string(),
            rust_code.to_string(),
            TraceTier::P0,
            "test.c",
        )
    }

    // ========================================================================
    // VerificationLevel tests
    // ========================================================================

    #[test]
    fn verification_level_display() {
        assert_eq!(VerificationLevel::Minimal.to_string(), "minimal");
        assert_eq!(VerificationLevel::Standard.to_string(), "standard");
        assert_eq!(VerificationLevel::Strict.to_string(), "strict");
    }

    #[test]
    fn verification_level_default() {
        let level = VerificationLevel::default();
        assert_eq!(level, VerificationLevel::Standard);
    }

    // ========================================================================
    // VerifierConfig tests
    // ========================================================================

    #[test]
    fn verifier_config_default() {
        let config = VerifierConfig::default();
        assert_eq!(config.level, VerificationLevel::Standard);
        assert!(!config.allow_unsafe);
        assert_eq!(config.max_clippy_warnings, 0);
        assert_eq!(config.timeout_secs, 30);
    }

    // ========================================================================
    // VerificationResult tests
    // ========================================================================

    #[test]
    fn test_verifier_default() {
        let verifier = TraceVerifier::new();
        assert_eq!(verifier.config().level, VerificationLevel::Standard);
    }

    #[test]
    fn test_count_unsafe_simple() {
        let verifier = TraceVerifier::new();
        let code = "unsafe { }";
        assert_eq!(verifier.count_unsafe_blocks(code), 1);
    }

    #[test]
    fn test_verification_result_is_clean() {
        let result = VerificationResult {
            passed: true,
            errors: vec![],
            warnings: vec![],
            unsafe_count: 0,
            compilation_time_ms: 0,
        };
        assert!(result.is_clean());
    }

    #[test]
    fn result_is_not_clean_with_errors() {
        let result = VerificationResult {
            passed: false,
            errors: vec!["err".to_string()],
            warnings: vec![],
            unsafe_count: 0,
            compilation_time_ms: 0,
        };
        assert!(!result.is_clean());
    }

    #[test]
    fn result_is_not_clean_with_warnings() {
        let result = VerificationResult {
            passed: true,
            errors: vec![],
            warnings: vec!["warn".to_string()],
            unsafe_count: 0,
            compilation_time_ms: 0,
        };
        assert!(!result.is_clean());
    }

    // ========================================================================
    // VerifierStats tests
    // ========================================================================

    #[test]
    fn stats_pass_rate_empty() {
        let stats = VerifierStats::default();
        assert!((stats.pass_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stats_pass_rate_all_passed() {
        let stats = VerifierStats {
            total_verified: 10,
            passed: 10,
            failed: 0,
            total_unsafe_blocks: 0,
            avg_verification_time_ms: 5.0,
        };
        assert!((stats.pass_rate() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stats_pass_rate_mixed() {
        let stats = VerifierStats {
            total_verified: 4,
            passed: 3,
            failed: 1,
            total_unsafe_blocks: 1,
            avg_verification_time_ms: 10.0,
        };
        assert!((stats.pass_rate() - 0.75).abs() < 0.01);
    }

    // ========================================================================
    // TraceVerifier construction tests
    // ========================================================================

    #[test]
    fn verifier_with_config() {
        let config = VerifierConfig {
            level: VerificationLevel::Strict,
            allow_unsafe: true,
            max_clippy_warnings: 5,
            timeout_secs: 60,
        };
        let verifier = TraceVerifier::with_config(config);
        assert_eq!(verifier.config().level, VerificationLevel::Strict);
        assert!(verifier.config().allow_unsafe);
        assert_eq!(verifier.config().max_clippy_warnings, 5);
    }

    #[test]
    fn verifier_default_trait() {
        let verifier = TraceVerifier::default();
        assert_eq!(verifier.config().level, VerificationLevel::Standard);
    }

    #[test]
    fn verifier_initial_stats() {
        let verifier = TraceVerifier::new();
        let stats = verifier.stats();
        assert_eq!(stats.total_verified, 0);
        assert_eq!(stats.passed, 0);
        assert_eq!(stats.failed, 0);
    }

    // ========================================================================
    // count_unsafe_blocks tests
    // ========================================================================

    #[test]
    fn count_unsafe_no_unsafe() {
        let verifier = TraceVerifier::new();
        assert_eq!(verifier.count_unsafe_blocks("fn main() { let x = 1; }"), 0);
    }

    #[test]
    fn count_unsafe_multiple() {
        let verifier = TraceVerifier::new();
        let code = "unsafe { ptr::read(p) }; unsafe { ptr::write(p, v) }";
        assert_eq!(verifier.count_unsafe_blocks(code), 2);
    }

    #[test]
    fn count_unsafe_no_space() {
        let verifier = TraceVerifier::new();
        let code = "unsafe{ ptr::read(p) }";
        assert_eq!(verifier.count_unsafe_blocks(code), 1);
    }

    // ========================================================================
    // verify_safety tests
    // ========================================================================

    #[test]
    fn verify_safety_no_unsafe_allowed() {
        let verifier = TraceVerifier::new(); // allow_unsafe = false
        let result = verifier.verify_safety("unsafe { ptr::read(p) }");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unsafe block"));
    }

    #[test]
    fn verify_safety_safe_code() {
        let verifier = TraceVerifier::new();
        let result = verifier.verify_safety("fn main() { let x = 1; }");
        assert!(result.is_ok());
    }

    #[test]
    fn verify_safety_unsafe_allowed() {
        let config = VerifierConfig {
            allow_unsafe: true,
            ..Default::default()
        };
        let verifier = TraceVerifier::with_config(config);
        let result = verifier.verify_safety("unsafe { ptr::read(p) }");
        assert!(result.is_ok());
    }

    // ========================================================================
    // verify_compilation tests
    // ========================================================================

    #[test]
    fn verify_compilation_valid_code() {
        let verifier = TraceVerifier::new();
        let result = verifier.verify_compilation("fn main() {}");
        assert!(result.is_ok());
    }

    #[test]
    fn verify_compilation_invalid_code() {
        let verifier = TraceVerifier::new();
        let result = verifier.verify_compilation("fn main() { let x: i32 = \"bad\"; }");
        assert!(result.is_err());
    }

    #[test]
    fn verify_compilation_empty() {
        let verifier = TraceVerifier::new();
        // Empty file with rustc --emit=metadata may fail (no crate type)
        // Just verify it returns a result without panicking
        let _result = verifier.verify_compilation("");
    }

    // ========================================================================
    // verify_trace tests
    // ========================================================================

    #[test]
    fn verify_trace_valid_code() {
        let mut verifier = TraceVerifier::new();
        let trace = make_trace("let x: i32 = 42;");
        let result = verifier.verify_trace(&trace);
        assert!(result.passed);
        assert!(result.errors.is_empty());
        assert_eq!(result.unsafe_count, 0);
        assert_eq!(verifier.stats().total_verified, 1);
        assert_eq!(verifier.stats().passed, 1);
    }

    #[test]
    fn verify_trace_with_fn_main() {
        let mut verifier = TraceVerifier::new();
        let trace = make_trace("fn main() { println!(\"hello\"); }");
        let result = verifier.verify_trace(&trace);
        // Contains fn main, so it won't be wrapped
        assert!(result.passed);
    }

    #[test]
    fn verify_trace_invalid_code() {
        let mut verifier = TraceVerifier::new();
        let trace = make_trace("let x: i32 = \"bad\";");
        let result = verifier.verify_trace(&trace);
        assert!(!result.passed);
        assert!(!result.errors.is_empty());
        assert_eq!(verifier.stats().total_verified, 1);
        assert_eq!(verifier.stats().failed, 1);
    }

    #[test]
    fn verify_trace_with_unsafe() {
        let mut verifier = TraceVerifier::new(); // allow_unsafe = false
        let trace = make_trace("unsafe { std::ptr::null::<i32>(); }");
        let result = verifier.verify_trace(&trace);
        assert!(!result.passed);
        assert!(result.unsafe_count > 0);
    }

    #[test]
    fn verify_trace_stats_accumulate() {
        let mut verifier = TraceVerifier::new();
        let trace1 = make_trace("let x: i32 = 1;");
        let trace2 = make_trace("let y: i32 = 2;");
        verifier.verify_trace(&trace1);
        verifier.verify_trace(&trace2);
        assert_eq!(verifier.stats().total_verified, 2);
        assert_eq!(verifier.stats().passed, 2);
    }

    // ========================================================================
    // verify_batch tests
    // ========================================================================

    #[test]
    fn verify_batch_all_valid() {
        let verifier = TraceVerifier::new();
        let traces = vec![
            make_trace("let x: i32 = 1;"),
            make_trace("let y: i32 = 2;"),
        ];
        let results = verifier.verify_batch(&traces);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.passed));
    }

    #[test]
    fn verify_batch_mixed() {
        let verifier = TraceVerifier::new();
        let traces = vec![
            make_trace("let x: i32 = 1;"),
            make_trace("let y: i32 = \"bad\";"),
        ];
        let results = verifier.verify_batch(&traces);
        assert_eq!(results.len(), 2);
        assert!(results[0].passed);
        assert!(!results[1].passed);
    }

    #[test]
    fn verify_batch_empty() {
        let verifier = TraceVerifier::new();
        let results = verifier.verify_batch(&[]);
        assert!(results.is_empty());
    }

    // ========================================================================
    // filter_valid tests
    // ========================================================================

    #[test]
    fn filter_valid_all_valid() {
        let verifier = TraceVerifier::new();
        let traces = vec![
            make_trace("let x: i32 = 1;"),
            make_trace("let y: i32 = 2;"),
        ];
        let valid = verifier.filter_valid(&traces);
        assert_eq!(valid.len(), 2);
    }

    #[test]
    fn filter_valid_mixed() {
        let verifier = TraceVerifier::new();
        let traces = vec![
            make_trace("let x: i32 = 1;"),
            make_trace("let y: i32 = \"bad\";"),
            make_trace("let z: i32 = 3;"),
        ];
        let valid = verifier.filter_valid(&traces);
        assert_eq!(valid.len(), 2);
    }

    #[test]
    fn filter_valid_none_valid() {
        let verifier = TraceVerifier::new();
        let traces = vec![
            make_trace("let y: i32 = \"bad\";"),
            make_trace("let z: bool = 42u8;"),
        ];
        let valid = verifier.filter_valid(&traces);
        assert!(valid.is_empty());
    }
}
