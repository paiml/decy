//! Trace Verifier Tests - DECY-108
//!
//! RED Phase: These tests define the contract for TraceVerifier
//! per unified spec Section 6.2 (Poka-Yoke verification gate).
//!
//! The verifier ensures only SAFE, COMPILING Rust enters the training dataset.
//!
//! Tests verify:
//! - Rust code compilation check
//! - Unsafe block detection
//! - Clippy warning detection
//! - GoldenTrace verification
//! - Verification statistics

use decy_oracle::golden_trace::{GoldenTrace, TraceTier};
use decy_oracle::trace_verifier::{
    TraceVerifier, VerificationLevel, VerificationResult, VerifierConfig, VerifierStats,
};

// ============================================================================
// VERIFIER CREATION TESTS
// ============================================================================

#[test]
fn test_verifier_creation_default() {
    let verifier = TraceVerifier::new();
    assert!(verifier.config().level == VerificationLevel::Standard);
}

#[test]
fn test_verifier_creation_with_config() {
    let config = VerifierConfig {
        level: VerificationLevel::Strict,
        allow_unsafe: false,
        max_clippy_warnings: 0,
        timeout_secs: 30,
    };
    let verifier = TraceVerifier::with_config(config);
    assert!(verifier.config().level == VerificationLevel::Strict);
    assert!(!verifier.config().allow_unsafe);
}

// ============================================================================
// VERIFICATION LEVEL TESTS
// ============================================================================

#[test]
fn test_verification_level_minimal() {
    // Minimal: Only check compilation
    let level = VerificationLevel::Minimal;
    assert_eq!(level.to_string(), "minimal");
}

#[test]
fn test_verification_level_standard() {
    // Standard: Compilation + unsafe check
    let level = VerificationLevel::Standard;
    assert_eq!(level.to_string(), "standard");
}

#[test]
fn test_verification_level_strict() {
    // Strict: Compilation + unsafe + clippy
    let level = VerificationLevel::Strict;
    assert_eq!(level.to_string(), "strict");
}

// ============================================================================
// RUST COMPILATION TESTS
// ============================================================================

#[test]
fn test_verify_valid_rust_compiles() {
    let verifier = TraceVerifier::new();
    let rust_code = r#"
fn main() {
    let x: i32 = 10;
    println!("{}", x);
}
"#;

    let result = verifier.verify_compilation(rust_code);
    assert!(result.is_ok());
}

#[test]
fn test_verify_invalid_rust_fails() {
    let verifier = TraceVerifier::new();
    let rust_code = r#"
fn main() {
    let x: i32 = "not an int";  // Type error
}
"#;

    let result = verifier.verify_compilation(rust_code);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("E0308") || err.contains("error"));
}

#[test]
fn test_verify_syntax_error_fails() {
    let verifier = TraceVerifier::new();
    let rust_code = r#"
fn main( {  // Missing paren
    let x = 10;
}
"#;

    let result = verifier.verify_compilation(rust_code);
    assert!(result.is_err());
}

// ============================================================================
// UNSAFE DETECTION TESTS
// ============================================================================

#[test]
fn test_detect_no_unsafe() {
    let verifier = TraceVerifier::new();
    let rust_code = r#"
fn safe_function() {
    let x = 10;
}
"#;

    let unsafe_count = verifier.count_unsafe_blocks(rust_code);
    assert_eq!(unsafe_count, 0);
}

#[test]
fn test_detect_single_unsafe() {
    let verifier = TraceVerifier::new();
    let rust_code = r#"
fn uses_unsafe() {
    unsafe {
        let ptr: *const i32 = std::ptr::null();
    }
}
"#;

    let unsafe_count = verifier.count_unsafe_blocks(rust_code);
    assert_eq!(unsafe_count, 1);
}

#[test]
fn test_detect_multiple_unsafe() {
    let verifier = TraceVerifier::new();
    let rust_code = r#"
fn uses_unsafe() {
    unsafe { let a = 1; }
    unsafe { let b = 2; }
    unsafe { let c = 3; }
}
"#;

    let unsafe_count = verifier.count_unsafe_blocks(rust_code);
    assert_eq!(unsafe_count, 3);
}

#[test]
fn test_reject_unsafe_when_not_allowed() {
    let config = VerifierConfig {
        level: VerificationLevel::Strict,
        allow_unsafe: false,
        max_clippy_warnings: 0,
        timeout_secs: 30,
    };
    let verifier = TraceVerifier::with_config(config);

    let rust_code = r#"
fn uses_unsafe() {
    unsafe { let ptr: *const i32 = std::ptr::null(); }
}
"#;

    let result = verifier.verify_safety(rust_code);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("unsafe"));
}

#[test]
fn test_allow_unsafe_when_configured() {
    let config = VerifierConfig {
        level: VerificationLevel::Minimal,
        allow_unsafe: true,
        max_clippy_warnings: 10,
        timeout_secs: 30,
    };
    let verifier = TraceVerifier::with_config(config);

    let rust_code = r#"
fn uses_unsafe() {
    unsafe { let ptr: *const i32 = std::ptr::null(); }
}
"#;

    let result = verifier.verify_safety(rust_code);
    assert!(result.is_ok());
}

// ============================================================================
// GOLDEN TRACE VERIFICATION TESTS
// ============================================================================

#[test]
fn test_verify_golden_trace_valid() {
    let mut verifier = TraceVerifier::new();

    let trace = GoldenTrace::new(
        "int x = 10;".to_string(),
        "let x: i32 = 10;".to_string(),
        TraceTier::P0,
        "test.c",
    );

    let result = verifier.verify_trace(&trace);
    assert!(result.passed);
    assert!(result.errors.is_empty());
}

#[test]
fn test_verify_golden_trace_invalid_rust() {
    let mut verifier = TraceVerifier::new();

    let trace = GoldenTrace::new(
        "int x = 10;".to_string(),
        "let x: i32 = \"not an int\";".to_string(), // Invalid Rust
        TraceTier::P0,
        "test.c",
    );

    let result = verifier.verify_trace(&trace);
    assert!(!result.passed);
    assert!(!result.errors.is_empty());
}

#[test]
fn test_verify_golden_trace_with_unsafe() {
    let config = VerifierConfig {
        level: VerificationLevel::Strict,
        allow_unsafe: false,
        max_clippy_warnings: 0,
        timeout_secs: 30,
    };
    let mut verifier = TraceVerifier::with_config(config);

    let trace = GoldenTrace::new(
        "int* ptr = (int*)0;".to_string(),
        "let ptr: *const i32 = unsafe { std::ptr::null() };".to_string(),
        TraceTier::P1,
        "test.c",
    );

    let result = verifier.verify_trace(&trace);
    assert!(!result.passed);
    assert!(result.errors.iter().any(|e| e.contains("unsafe")));
}

// ============================================================================
// VERIFICATION RESULT TESTS
// ============================================================================

#[test]
fn test_verification_result_passed() {
    let result = VerificationResult {
        passed: true,
        errors: vec![],
        warnings: vec![],
        unsafe_count: 0,
        compilation_time_ms: 100,
    };

    assert!(result.passed);
    assert!(result.is_clean());
}

#[test]
fn test_verification_result_failed() {
    let result = VerificationResult {
        passed: false,
        errors: vec!["E0308: type mismatch".to_string()],
        warnings: vec![],
        unsafe_count: 0,
        compilation_time_ms: 100,
    };

    assert!(!result.passed);
    assert!(!result.is_clean());
}

#[test]
fn test_verification_result_with_warnings() {
    let result = VerificationResult {
        passed: true,
        errors: vec![],
        warnings: vec!["unused variable".to_string()],
        unsafe_count: 0,
        compilation_time_ms: 100,
    };

    assert!(result.passed);
    assert!(!result.is_clean()); // Has warnings
}

// ============================================================================
// VERIFIER STATS TESTS
// ============================================================================

#[test]
fn test_verifier_stats_initial() {
    let verifier = TraceVerifier::new();
    let stats = verifier.stats();

    assert_eq!(stats.total_verified, 0);
    assert_eq!(stats.passed, 0);
    assert_eq!(stats.failed, 0);
}

#[test]
fn test_verifier_stats_after_verification() {
    let mut verifier = TraceVerifier::new();

    // Verify a valid trace
    let valid_trace = GoldenTrace::new(
        "int x = 10;".to_string(),
        "let x: i32 = 10;".to_string(),
        TraceTier::P0,
        "test.c",
    );
    verifier.verify_trace(&valid_trace);

    // Verify an invalid trace
    let invalid_trace = GoldenTrace::new(
        "int x = 10;".to_string(),
        "let x: i32 = \"bad\";".to_string(),
        TraceTier::P0,
        "test.c",
    );
    verifier.verify_trace(&invalid_trace);

    let stats = verifier.stats();
    assert_eq!(stats.total_verified, 2);
    assert_eq!(stats.passed, 1);
    assert_eq!(stats.failed, 1);
}

#[test]
fn test_verifier_stats_pass_rate() {
    let stats = VerifierStats {
        total_verified: 100,
        passed: 80,
        failed: 20,
        total_unsafe_blocks: 5,
        avg_verification_time_ms: 50.0,
    };

    assert!((stats.pass_rate() - 0.8).abs() < 0.01);
}

// ============================================================================
// BATCH VERIFICATION TESTS
// ============================================================================

#[test]
fn test_verify_batch_all_valid() {
    let verifier = TraceVerifier::new();

    let traces = vec![
        GoldenTrace::new(
            "int a;".to_string(),
            "let a: i32;".to_string(),
            TraceTier::P0,
            "a.c",
        ),
        GoldenTrace::new(
            "int b;".to_string(),
            "let b: i32;".to_string(),
            TraceTier::P0,
            "b.c",
        ),
    ];

    let results = verifier.verify_batch(&traces);
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.passed));
}

#[test]
fn test_verify_batch_filters_invalid() {
    let verifier = TraceVerifier::new();

    let traces = vec![
        GoldenTrace::new(
            "int a;".to_string(),
            "let a: i32;".to_string(),
            TraceTier::P0,
            "a.c",
        ),
        GoldenTrace::new(
            "int b;".to_string(),
            "let b: i32 = \"bad\";".to_string(),
            TraceTier::P0,
            "b.c",
        ),
        GoldenTrace::new(
            "int c;".to_string(),
            "let c: i32;".to_string(),
            TraceTier::P0,
            "c.c",
        ),
    ];

    let valid_traces = verifier.filter_valid(&traces);
    assert_eq!(valid_traces.len(), 2);
}

// ============================================================================
// TRACE VERIFICATION: MAIN WRAPPING TESTS
// ============================================================================

#[test]
fn test_verify_trace_wraps_code_without_fn_main() {
    // verify_trace wraps code not containing "fn main" in main() block
    let mut verifier = TraceVerifier::new();

    let trace = GoldenTrace::new(
        "int x = 42;".to_string(),
        "let x: i32 = 42;".to_string(),
        TraceTier::P0,
        "test.c",
    );

    let result = verifier.verify_trace(&trace);
    // Should pass because code is wrapped in fn main() { ... }
    assert!(result.passed, "Errors: {:?}", result.errors);
}

#[test]
fn test_verify_trace_no_wrap_when_has_fn_main() {
    // verify_trace uses code as-is when it contains "fn main"
    let mut verifier = TraceVerifier::new();

    let trace = GoldenTrace::new(
        "int main() { return 0; }".to_string(),
        "fn main() { let _x: i32 = 0; }".to_string(),
        TraceTier::P0,
        "test.c",
    );

    let result = verifier.verify_trace(&trace);
    assert!(result.passed, "Errors: {:?}", result.errors);
}

// ============================================================================
// TRACE VERIFICATION: UNSAFE BLOCK COUNTING
// ============================================================================

#[test]
fn test_verify_trace_unsafe_rejected_in_strict_mode() {
    let config = VerifierConfig {
        level: VerificationLevel::Strict,
        allow_unsafe: false,
        max_clippy_warnings: 0,
        timeout_secs: 30,
    };
    let mut verifier = TraceVerifier::with_config(config);

    let trace = GoldenTrace::new(
        "int* p = NULL;".to_string(),
        "fn main() { unsafe { let _p: *const i32 = std::ptr::null(); } }".to_string(),
        TraceTier::P1,
        "test.c",
    );

    let result = verifier.verify_trace(&trace);
    assert!(!result.passed);
    assert!(result.unsafe_count > 0);
    assert!(result.errors.iter().any(|e| e.contains("unsafe")));
}

#[test]
fn test_verify_trace_unsafe_allowed_in_permissive_config() {
    let config = VerifierConfig {
        level: VerificationLevel::Minimal,
        allow_unsafe: true,
        max_clippy_warnings: 10,
        timeout_secs: 30,
    };
    let mut verifier = TraceVerifier::with_config(config);

    let trace = GoldenTrace::new(
        "int* p = NULL;".to_string(),
        "fn main() { unsafe { let _p: *const i32 = std::ptr::null(); } }".to_string(),
        TraceTier::P1,
        "test.c",
    );

    let result = verifier.verify_trace(&trace);
    assert!(result.passed, "Errors: {:?}", result.errors);
    assert!(result.unsafe_count > 0);
}

// ============================================================================
// STATS TRACKING TESTS
// ============================================================================

#[test]
fn test_verifier_stats_increments_on_pass() {
    let mut verifier = TraceVerifier::new();

    let trace = GoldenTrace::new(
        "int x = 1;".to_string(),
        "let _x: i32 = 1;".to_string(),
        TraceTier::P0,
        "test.c",
    );

    verifier.verify_trace(&trace);

    let stats = verifier.stats();
    assert_eq!(stats.total_verified, 1);
    assert_eq!(stats.passed, 1);
    assert_eq!(stats.failed, 0);
}

#[test]
fn test_verifier_stats_increments_on_fail() {
    let mut verifier = TraceVerifier::new();

    let trace = GoldenTrace::new(
        "int x;".to_string(),
        "let x: i32 = \"bad\";".to_string(),
        TraceTier::P0,
        "test.c",
    );

    verifier.verify_trace(&trace);

    let stats = verifier.stats();
    assert_eq!(stats.total_verified, 1);
    assert_eq!(stats.passed, 0);
    assert_eq!(stats.failed, 1);
}

#[test]
fn test_verifier_stats_tracks_unsafe_blocks() {
    let config = VerifierConfig {
        level: VerificationLevel::Minimal,
        allow_unsafe: true,
        max_clippy_warnings: 10,
        timeout_secs: 30,
    };
    let mut verifier = TraceVerifier::with_config(config);

    let trace = GoldenTrace::new(
        "int* p;".to_string(),
        "fn main() { unsafe { let _p: *const i32 = std::ptr::null(); } }".to_string(),
        TraceTier::P1,
        "test.c",
    );

    verifier.verify_trace(&trace);

    let stats = verifier.stats();
    assert!(stats.total_unsafe_blocks > 0);
}

#[test]
fn test_verifier_stats_avg_time_multiple_traces() {
    let mut verifier = TraceVerifier::new();

    // Verify multiple traces to test average time calculation
    for i in 0..3 {
        let trace = GoldenTrace::new(
            format!("int x{} = {};", i, i),
            format!("let _x{}: i32 = {};", i, i),
            TraceTier::P0,
            "test.c",
        );
        verifier.verify_trace(&trace);
    }

    let stats = verifier.stats();
    assert_eq!(stats.total_verified, 3);
    assert_eq!(stats.passed, 3);
    assert!(stats.avg_verification_time_ms > 0.0);
}

// ============================================================================
// COMPILATION: ADDITIONAL PATH COVERAGE
// ============================================================================

#[test]
fn test_verify_compilation_with_complex_valid_code() {
    let verifier = TraceVerifier::new();
    let rust_code = r#"
fn main() {
    let mut v: Vec<i32> = Vec::new();
    v.push(1);
    v.push(2);
    let sum: i32 = v.iter().sum();
    println!("Sum: {}", sum);
}
"#;

    let result = verifier.verify_compilation(rust_code);
    assert!(result.is_ok());
}

#[test]
fn test_verify_compilation_lifetime_error() {
    let verifier = TraceVerifier::new();
    let rust_code = r#"
fn main() {
    let r;
    {
        let x = 5;
        r = &x;
    }
    println!("{}", r);
}
"#;

    let result = verifier.verify_compilation(rust_code);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("error") || err.contains("borrow"),
        "Got: {}",
        err
    );
}

// ============================================================================
// STATS: ZERO DIVISION EDGE CASE
// ============================================================================

#[test]
fn test_verifier_stats_pass_rate_zero_total() {
    let stats = VerifierStats {
        total_verified: 0,
        passed: 0,
        failed: 0,
        total_unsafe_blocks: 0,
        avg_verification_time_ms: 0.0,
    };

    // Should return 0.0, not panic on division by zero
    assert!((stats.pass_rate() - 0.0).abs() < 0.001);
}

// ============================================================================
// DEFAULT TRAIT IMPLEMENTATIONS
// ============================================================================

#[test]
fn test_verifier_default_trait() {
    let verifier = TraceVerifier::default();
    assert_eq!(verifier.config().level, VerificationLevel::Standard);
    assert!(!verifier.config().allow_unsafe);
}

#[test]
fn test_verifier_config_default_trait() {
    let config = VerifierConfig::default();
    assert_eq!(config.level, VerificationLevel::Standard);
    assert!(!config.allow_unsafe);
    assert_eq!(config.max_clippy_warnings, 0);
    assert_eq!(config.timeout_secs, 30);
}

#[test]
fn test_verification_level_default_trait() {
    let level = VerificationLevel::default();
    assert_eq!(level, VerificationLevel::Standard);
}

// ============================================================================
// UNSAFE COUNTING EDGE CASES
// ============================================================================

#[test]
fn test_count_unsafe_no_space_variant() {
    let verifier = TraceVerifier::new();
    // "unsafe{" without space — also counted
    let code = "unsafe{ let ptr = std::ptr::null::<i32>(); }";
    assert_eq!(verifier.count_unsafe_blocks(code), 1);
}

#[test]
fn test_count_unsafe_mixed_variants() {
    let verifier = TraceVerifier::new();
    let code = "unsafe { a(); } unsafe{ b(); }";
    assert_eq!(verifier.count_unsafe_blocks(code), 2);
}

// ============================================================================
// VERIFY SAFETY EDGE CASES
// ============================================================================

#[test]
fn test_verify_safety_safe_code_passes() {
    let verifier = TraceVerifier::new();
    let result = verifier.verify_safety("fn safe() { let x = 1; }");
    assert!(result.is_ok());
}

// ============================================================================
// VERIFY BATCH/FILTER EMPTY INPUT
// ============================================================================

#[test]
fn test_verify_batch_empty() {
    let verifier = TraceVerifier::new();
    let results = verifier.verify_batch(&[]);
    assert!(results.is_empty());
}

#[test]
fn test_filter_valid_empty() {
    let verifier = TraceVerifier::new();
    let valid = verifier.filter_valid(&[]);
    assert!(valid.is_empty());
}
