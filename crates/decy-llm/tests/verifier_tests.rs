//! Tests for verification and iteration framework (DECY-100).
//!
//! Verifies that generated code is compiled, tested, and iterated on failures.

use decy_llm::{CodeVerifier, GeneratedCode, IterationContext, VerificationLoop, VerificationResult};

// ============================================================================
// TEST 1: Create success result
// ============================================================================

#[test]
fn test_verification_success_result() {
    let result = VerificationResult::success();
    assert!(result.success);
    assert!(result.compiles);
    assert!(result.tests_pass);
    assert!(result.compile_errors.is_empty());
}

// ============================================================================
// TEST 2: Create compile failure result
// ============================================================================

#[test]
fn test_compile_failure_result() {
    let errors = vec!["error[E0308]: mismatched types".to_string()];
    let result = VerificationResult::compile_failure(errors);

    assert!(!result.success);
    assert!(!result.compiles);
    assert!(!result.compile_errors.is_empty());
}

// ============================================================================
// TEST 3: Create test failure result
// ============================================================================

#[test]
fn test_test_failure_result() {
    let failures = vec!["test_add failed: assertion failed".to_string()];
    let result = VerificationResult::test_failure(failures);

    assert!(!result.success);
    assert!(result.compiles); // Compiled but tests failed
    assert!(!result.tests_pass);
    assert!(!result.test_failures.is_empty());
}

// ============================================================================
// TEST 4: Create iteration context
// ============================================================================

#[test]
fn test_create_iteration_context() {
    let ctx = IterationContext::new(3);

    assert_eq!(ctx.iteration, 1);
    assert_eq!(ctx.max_iterations, 3);
    assert!(ctx.previous_code.is_none());
    assert!(ctx.previous_errors.is_empty());
}

// ============================================================================
// TEST 5: Iteration context can_retry
// ============================================================================

#[test]
fn test_iteration_can_retry() {
    let mut ctx = IterationContext::new(3);

    assert!(ctx.can_retry()); // iteration 1 of 3

    ctx.record_failure("fn bad() {}", vec!["error".to_string()]);
    assert!(ctx.can_retry()); // iteration 2 of 3

    ctx.record_failure("fn bad() {}", vec!["error".to_string()]);
    assert!(ctx.can_retry()); // iteration 3 of 3

    ctx.record_failure("fn bad() {}", vec!["error".to_string()]);
    assert!(!ctx.can_retry()); // iteration 4 of 3 - exceeded
}

// ============================================================================
// TEST 6: Iteration context records failures
// ============================================================================

#[test]
fn test_iteration_records_failure() {
    let mut ctx = IterationContext::new(5);

    ctx.record_failure("fn attempt1() {}", vec!["error 1".to_string()]);

    assert_eq!(ctx.iteration, 2);
    assert!(ctx.previous_code.as_ref().unwrap().contains("attempt1"));
    assert!(ctx.previous_errors.contains(&"error 1".to_string()));
}

// ============================================================================
// TEST 7: Iteration context generates feedback
// ============================================================================

#[test]
fn test_iteration_get_feedback() {
    let mut ctx = IterationContext::new(3);

    ctx.record_failure(
        "fn broken() { x }",
        vec!["error[E0425]: cannot find value `x`".to_string()],
    );

    let feedback = ctx.get_feedback();

    assert!(feedback.contains("error") || feedback.contains("E0425"));
}

// ============================================================================
// TEST 8: Create code verifier
// ============================================================================

#[test]
fn test_create_code_verifier() {
    let _verifier = CodeVerifier::new();
    // Just verify creation doesn't panic
}

// ============================================================================
// TEST 9: Verification loop checks success
// ============================================================================

#[test]
fn test_verification_loop_success_check() {
    let loop_runner = VerificationLoop::new(3);

    let success = VerificationResult::success();
    assert!(loop_runner.is_success(&success));

    let failure = VerificationResult::compile_failure(vec!["error".to_string()]);
    assert!(!loop_runner.is_success(&failure));
}

// ============================================================================
// TEST 10: Verification loop formats feedback
// ============================================================================

#[test]
fn test_verification_loop_format_feedback() {
    let loop_runner = VerificationLoop::new(3);

    let failure = VerificationResult::compile_failure(vec![
        "error[E0308]: mismatched types".to_string(),
        "expected i32, found &str".to_string(),
    ]);

    let feedback = loop_runner.format_feedback(&failure);

    assert!(feedback.contains("E0308") || feedback.contains("mismatched"));
}
