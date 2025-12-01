//! Minimal tests for verifier module.

use crate::llm_codegen::GeneratedCode;
use crate::verifier::*;

fn code(s: &str) -> GeneratedCode {
    GeneratedCode {
        code: s.into(),
        confidence: 0.9,
        reasoning: "test".into(),
        warnings: vec![],
    }
}

#[test]
fn result_success() {
    let r = VerificationResult::success();
    assert!(r.success && r.compiles && r.tests_pass);
}

#[test]
fn result_compile_failure() {
    let r = VerificationResult::compile_failure(vec!["err".into()]);
    assert!(!r.success && !r.compiles);
}

#[test]
fn result_test_failure() {
    let r = VerificationResult::test_failure(vec!["fail".into()]);
    assert!(!r.success && r.compiles && !r.tests_pass);
}

#[test]
fn iteration_new() {
    let ctx = IterationContext::new(3);
    assert_eq!(ctx.iteration, 1);
    assert!(ctx.can_retry());
}

#[test]
fn iteration_past_max() {
    let mut ctx = IterationContext::new(2);
    ctx.iteration = 3;
    assert!(!ctx.can_retry());
}

#[test]
fn iteration_record_failure() {
    let mut ctx = IterationContext::new(5);
    ctx.record_failure("code", vec!["e1".into()]);
    assert_eq!(ctx.iteration, 2);
    assert_eq!(ctx.previous_code, Some("code".into()));
}

#[test]
fn iteration_feedback() {
    let mut ctx = IterationContext::new(3);
    ctx.record_failure("fn x()", vec!["error".into()]);
    let fb = ctx.get_feedback();
    assert!(fb.contains("Previous Errors") && fb.contains("Previous Code"));
}

#[test]
fn verifier_valid() {
    let v = CodeVerifier::new();
    let r = v.verify(&code("fn main() {}")).unwrap();
    assert!(r.success);
}

#[test]
fn verifier_empty() {
    let v = CodeVerifier::new();
    let r = v.verify(&code("")).unwrap();
    assert!(!r.success);
}

#[test]
fn verifier_unbalanced() {
    let v = CodeVerifier::new();
    let r = v.verify(&code("fn main() {")).unwrap();
    assert!(!r.compiles);
}

#[test]
fn compile_empty() {
    let v = CodeVerifier::new();
    assert!(v.compile("").is_err());
}

#[test]
fn lint_clean() {
    let v = CodeVerifier::new();
    assert_eq!(v.lint("let x = 5;").unwrap(), 0);
}

#[test]
fn lint_unwrap() {
    let v = CodeVerifier::new();
    assert_eq!(v.lint("x.unwrap()").unwrap(), 1);
}

#[test]
fn lint_multiple() {
    let v = CodeVerifier::new();
    assert_eq!(v.lint("x.unwrap(); panic!()").unwrap(), 2);
}

#[test]
fn loop_new() {
    let l = VerificationLoop::new(5);
    assert_eq!(l.max_iterations(), 5);
}

#[test]
fn loop_default() {
    let l = VerificationLoop::default();
    assert_eq!(l.max_iterations(), 3);
}

#[test]
fn loop_is_success() {
    let l = VerificationLoop::new(3);
    assert!(l.is_success(&VerificationResult::success()));
    assert!(!l.is_success(&VerificationResult::compile_failure(vec![])));
}

#[test]
fn loop_format_feedback() {
    let l = VerificationLoop::new(3);
    let r = VerificationResult::compile_failure(vec!["E0308".into()]);
    let fb = l.format_feedback(&r);
    assert!(fb.contains("Compilation Errors"));
}

#[test]
fn result_serde() {
    let r = VerificationResult::success();
    let json = serde_json::to_string(&r).unwrap();
    let r2: VerificationResult = serde_json::from_str(&json).unwrap();
    assert_eq!(r.success, r2.success);
}

#[test]
fn context_serde() {
    let ctx = IterationContext::new(5);
    let json = serde_json::to_string(&ctx).unwrap();
    let ctx2: IterationContext = serde_json::from_str(&json).unwrap();
    assert_eq!(ctx.max_iterations, ctx2.max_iterations);
}

// ============================================================================
// DECY-ML-004: "COMPILES ON FIRST TRY" METRIC TESTS
// ============================================================================

#[test]
fn compilation_metrics_new() {
    let metrics = CompilationMetrics::new();
    assert_eq!(metrics.total_attempts(), 0);
    assert_eq!(metrics.first_try_successes(), 0);
}

#[test]
fn compilation_metrics_record_first_try_success() {
    let mut metrics = CompilationMetrics::new();
    metrics.record_attempt(true, 1);
    assert_eq!(metrics.total_attempts(), 1);
    assert_eq!(metrics.first_try_successes(), 1);
    assert!((metrics.first_try_rate() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn compilation_metrics_record_retry_success() {
    let mut metrics = CompilationMetrics::new();
    metrics.record_attempt(true, 2); // Success on second try
    assert_eq!(metrics.total_attempts(), 1);
    assert_eq!(metrics.first_try_successes(), 0);
    assert!((metrics.first_try_rate() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn compilation_metrics_record_failure() {
    let mut metrics = CompilationMetrics::new();
    metrics.record_attempt(false, 3); // Failed after 3 attempts
    assert_eq!(metrics.total_attempts(), 1);
    assert_eq!(metrics.first_try_successes(), 0);
}

#[test]
fn compilation_metrics_rate_calculation() {
    let mut metrics = CompilationMetrics::new();
    // 8 first-try successes, 2 retries = 80% rate
    for _ in 0..8 {
        metrics.record_attempt(true, 1);
    }
    for _ in 0..2 {
        metrics.record_attempt(true, 2);
    }
    assert_eq!(metrics.total_attempts(), 10);
    assert_eq!(metrics.first_try_successes(), 8);
    assert!((metrics.first_try_rate() - 0.8).abs() < 0.001);
}

#[test]
fn compilation_metrics_meets_target() {
    let mut metrics = CompilationMetrics::new();
    // 85% first-try success rate meets target
    for _ in 0..85 {
        metrics.record_attempt(true, 1);
    }
    for _ in 0..15 {
        metrics.record_attempt(true, 2);
    }
    assert!(metrics.meets_target(0.85));
    assert!(!metrics.meets_target(0.90));
}

#[test]
fn compilation_metrics_average_iterations() {
    let mut metrics = CompilationMetrics::new();
    metrics.record_attempt(true, 1); // 1 iteration
    metrics.record_attempt(true, 2); // 2 iterations
    metrics.record_attempt(true, 3); // 3 iterations
    // Average: (1 + 2 + 3) / 3 = 2.0
    assert!((metrics.average_iterations() - 2.0).abs() < 0.001);
}

#[test]
fn compilation_metrics_zero_attempts() {
    let metrics = CompilationMetrics::new();
    // Should handle divide-by-zero gracefully
    assert!((metrics.first_try_rate() - 0.0).abs() < f64::EPSILON);
    assert!((metrics.average_iterations() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn compilation_metrics_serialize() {
    let mut metrics = CompilationMetrics::new();
    metrics.record_attempt(true, 1);
    let json = serde_json::to_string(&metrics).unwrap();
    assert!(json.contains("total_attempts"));
    assert!(json.contains("first_try_successes"));
}

#[test]
fn compilation_metrics_deserialize() {
    let mut metrics = CompilationMetrics::new();
    metrics.record_attempt(true, 1);
    metrics.record_attempt(true, 2);
    let json = serde_json::to_string(&metrics).unwrap();
    let parsed: CompilationMetrics = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.total_attempts(), 2);
    assert_eq!(parsed.first_try_successes(), 1);
}

#[test]
fn compilation_metrics_target_constant() {
    // Spec: 85% target rate
    assert!((CompilationMetrics::TARGET_RATE - 0.85).abs() < 0.001);
}

#[test]
fn compilation_metrics_iteration_histogram() {
    let mut metrics = CompilationMetrics::new();
    metrics.record_attempt(true, 1);
    metrics.record_attempt(true, 1);
    metrics.record_attempt(true, 2);
    metrics.record_attempt(true, 3);

    let histogram = metrics.iteration_histogram();
    assert_eq!(histogram.get(&1), Some(&2)); // 2 first-try successes
    assert_eq!(histogram.get(&2), Some(&1)); // 1 second-try success
    assert_eq!(histogram.get(&3), Some(&1)); // 1 third-try success
}

#[test]
fn compilation_metrics_reset() {
    let mut metrics = CompilationMetrics::new();
    metrics.record_attempt(true, 1);
    metrics.record_attempt(true, 2);
    metrics.reset();
    assert_eq!(metrics.total_attempts(), 0);
    assert_eq!(metrics.first_try_successes(), 0);
}
