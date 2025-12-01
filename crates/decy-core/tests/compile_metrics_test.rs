//! Integration tests for compile success rate metrics
//!
//! **Ticket**: DECY-181 - Add compile success rate metrics
//!
//! These tests verify that the transpilation pipeline tracks compile success rate
//! to enable measurement toward 80% single-shot compile target.

use decy_core::{transpile, CompileMetrics};

// ============================================================================
// DECY-181: Compile Metrics Tests (RED Phase)
// ============================================================================

#[test]
fn test_compile_metrics_new() {
    // DECY-181: Metrics should start with zero counts
    let metrics = CompileMetrics::new();

    assert_eq!(metrics.total_attempts(), 0);
    assert_eq!(metrics.successes(), 0);
    assert_eq!(metrics.failures(), 0);
    assert_eq!(metrics.success_rate(), 0.0);
}

#[test]
fn test_compile_metrics_record_success() {
    // DECY-181: Recording success should update metrics
    let mut metrics = CompileMetrics::new();

    metrics.record_success();

    assert_eq!(metrics.total_attempts(), 1);
    assert_eq!(metrics.successes(), 1);
    assert_eq!(metrics.failures(), 0);
    assert_eq!(metrics.success_rate(), 1.0);
}

#[test]
fn test_compile_metrics_record_failure() {
    // DECY-181: Recording failure should update metrics
    let mut metrics = CompileMetrics::new();

    metrics.record_failure("E0308: mismatched types");

    assert_eq!(metrics.total_attempts(), 1);
    assert_eq!(metrics.successes(), 0);
    assert_eq!(metrics.failures(), 1);
    assert_eq!(metrics.success_rate(), 0.0);
}

#[test]
fn test_compile_metrics_mixed_results() {
    // DECY-181: Mixed results should calculate correct rate
    let mut metrics = CompileMetrics::new();

    metrics.record_success();
    metrics.record_success();
    metrics.record_success();
    metrics.record_success();
    metrics.record_failure("E0308");

    assert_eq!(metrics.total_attempts(), 5);
    assert_eq!(metrics.successes(), 4);
    assert_eq!(metrics.failures(), 1);
    assert!((metrics.success_rate() - 0.8).abs() < 0.001);
}

#[test]
fn test_compile_metrics_meets_target() {
    // DECY-181: Should check against 80% target
    let mut metrics = CompileMetrics::new();

    // 4/5 = 80%
    for _ in 0..4 {
        metrics.record_success();
    }
    metrics.record_failure("E0308");

    assert!(metrics.meets_target(0.80));
    assert!(!metrics.meets_target(0.85));
}

#[test]
fn test_compile_metrics_error_tracking() {
    // DECY-181: Should track error codes for analysis
    let mut metrics = CompileMetrics::new();

    metrics.record_failure("E0308: mismatched types");
    metrics.record_failure("E0308: mismatched types");
    metrics.record_failure("E0502: cannot borrow");

    let errors = metrics.error_histogram();
    assert_eq!(errors.get("E0308"), Some(&2));
    assert_eq!(errors.get("E0502"), Some(&1));
}

#[test]
fn test_compile_metrics_reset() {
    // DECY-181: Reset should clear all metrics
    let mut metrics = CompileMetrics::new();

    metrics.record_success();
    metrics.record_failure("E0308");
    metrics.reset();

    assert_eq!(metrics.total_attempts(), 0);
    assert_eq!(metrics.successes(), 0);
    assert_eq!(metrics.failures(), 0);
}

#[test]
fn test_transpilation_result_with_metrics() {
    // DECY-181: TranspilationResult should include compile verification
    let c_code = r#"
int add(int a, int b) {
    return a + b;
}
"#;

    let result = transpile(c_code).expect("Transpilation should succeed");

    // Result should be valid Rust code
    assert!(result.contains("fn add"));
}

#[test]
fn test_transpile_with_verification() {
    // DECY-181: transpile_with_verification should return verification result
    let c_code = r#"
int factorial(int n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}
"#;

    let result = decy_core::transpile_with_verification(c_code);
    assert!(result.is_ok());

    let verification = result.unwrap();
    // Should have verification status
    assert!(verification.rust_code.contains("fn factorial"));
}

#[test]
fn test_metrics_to_markdown() {
    // DECY-181: Should generate markdown report
    let mut metrics = CompileMetrics::new();

    for _ in 0..8 {
        metrics.record_success();
    }
    metrics.record_failure("E0308");
    metrics.record_failure("E0502");

    let report = metrics.to_markdown();
    assert!(report.contains("Success Rate"));
    assert!(report.contains("80%") || report.contains("0.80"));
}

#[test]
fn test_metrics_to_json() {
    // DECY-181: Should serialize to JSON for CI integration
    let mut metrics = CompileMetrics::new();

    metrics.record_success();
    metrics.record_failure("E0308");

    let json = metrics.to_json();
    assert!(json.contains("total_attempts"));
    assert!(json.contains("successes"));
    assert!(json.contains("failures"));
    // JSON has the raw counts, not computed rate
    assert!(json.contains("\"successes\": 1"));
}
