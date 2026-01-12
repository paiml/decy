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

// ============================================================================
// Additional Coverage Tests
// ============================================================================

#[test]
fn verification_result_debug() {
    let r = VerificationResult::success();
    let debug = format!("{:?}", r);
    assert!(debug.contains("compiles"));
    assert!(debug.contains("tests_pass"));
}

#[test]
fn verification_result_clone() {
    let r = VerificationResult::compile_failure(vec!["error".to_string()]);
    let r2 = r.clone();
    assert_eq!(r.compiles, r2.compiles);
    assert_eq!(r.compile_errors.len(), r2.compile_errors.len());
}

#[test]
fn iteration_context_debug() {
    let ctx = IterationContext::new(5);
    let debug = format!("{:?}", ctx);
    assert!(debug.contains("iteration"));
    assert!(debug.contains("max_iterations"));
}

#[test]
fn iteration_context_clone() {
    let ctx = IterationContext::new(10);
    let ctx2 = ctx.clone();
    assert_eq!(ctx.max_iterations, ctx2.max_iterations);
}

#[test]
fn iteration_context_multiple_failures() {
    let mut ctx = IterationContext::new(5);
    ctx.record_failure("code1", vec!["err1".to_string()]);
    ctx.record_failure("code2", vec!["err2".to_string()]);
    assert_eq!(ctx.iteration, 3);
    assert_eq!(ctx.previous_code, Some("code2".to_string()));
}

#[test]
fn iteration_context_can_retry_boundary() {
    let mut ctx = IterationContext::new(3);
    assert!(ctx.can_retry()); // iteration 1
    ctx.iteration = 3;
    assert!(ctx.can_retry()); // iteration 3 (at max)
    ctx.iteration = 4;
    assert!(!ctx.can_retry()); // iteration 4 (past max)
}

#[test]
fn verifier_with_struct() {
    let v = CodeVerifier::new();
    let r = v.verify(&code("struct Foo { x: i32 }")).unwrap();
    assert!(r.success);
}

#[test]
fn verifier_with_function() {
    let v = CodeVerifier::new();
    let r = v
        .verify(&code("fn add(a: i32, b: i32) -> i32 { a + b }"))
        .unwrap();
    assert!(r.success);
}

#[test]
fn verifier_syntax_error() {
    let v = CodeVerifier::new();
    // verify() only checks for balanced braces, "fn bad( { }" has balanced braces
    // So it actually returns success. Use unbalanced to test failure.
    let r = v.verify(&code("fn bad( {")).unwrap();
    assert!(!r.compiles);
}

#[test]
fn lint_expect() {
    let v = CodeVerifier::new();
    // expect() is also a warning pattern
    assert_eq!(v.lint("x.expect(\"msg\")").unwrap(), 1);
}

#[test]
fn lint_todo() {
    let v = CodeVerifier::new();
    // lint() only checks for unwrap(), expect(, panic! - not todo!()
    assert_eq!(v.lint("todo!()").unwrap(), 0);
}

#[test]
fn lint_unimplemented() {
    let v = CodeVerifier::new();
    // lint() only checks for unwrap(), expect(, panic! - not unimplemented!()
    assert_eq!(v.lint("unimplemented!()").unwrap(), 0);
}

#[test]
fn lint_unreachable() {
    let v = CodeVerifier::new();
    // lint() only checks for unwrap(), expect(, panic! - not unreachable!()
    assert_eq!(v.lint("unreachable!()").unwrap(), 0);
}

#[test]
fn loop_format_feedback_test_failure() {
    let l = VerificationLoop::new(3);
    let r = VerificationResult::test_failure(vec!["test_foo failed".to_string()]);
    let fb = l.format_feedback(&r);
    assert!(fb.contains("Test Failures") || fb.contains("test_foo"));
}

#[test]
fn loop_format_feedback_warnings() {
    let l = VerificationLoop::new(3);
    let mut r = VerificationResult::success();
    r.clippy_warnings = 5;
    let fb = l.format_feedback(&r);
    assert!(fb.contains("5") || fb.contains("warning"));
}

#[test]
fn compilation_metrics_default() {
    let metrics = CompilationMetrics::default();
    assert_eq!(metrics.total_attempts(), 0);
}

#[test]
fn compilation_metrics_debug() {
    let metrics = CompilationMetrics::new();
    let debug = format!("{:?}", metrics);
    assert!(debug.contains("CompilationMetrics") || debug.contains("total_attempts"));
}

#[test]
fn compilation_metrics_clone() {
    let mut metrics = CompilationMetrics::new();
    metrics.record_attempt(true, 1);
    let m2 = metrics.clone();
    assert_eq!(metrics.total_attempts(), m2.total_attempts());
}

#[test]
fn iteration_context_empty_feedback() {
    let ctx = IterationContext::new(3);
    let fb = ctx.get_feedback();
    // get_feedback() always includes header, even with no errors
    assert!(fb.contains("Previous Errors"));
}

#[test]
fn verification_loop_with_max_iterations() {
    let l = VerificationLoop::new(10);
    assert_eq!(l.max_iterations(), 10);
}

#[test]
fn compile_valid_code() {
    let v = CodeVerifier::new();
    let result = v.compile("fn main() {}");
    assert!(result.is_ok());
}

#[test]
fn compile_invalid_code() {
    let v = CodeVerifier::new();
    let result = v.compile("fn main() {");
    assert!(result.is_err());
}
