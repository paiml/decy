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
