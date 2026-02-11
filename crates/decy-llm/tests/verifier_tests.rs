//! Tests for verification and iteration framework (DECY-100).
//!
//! Verifies that generated code is compiled, tested, and iterated on failures.

use decy_llm::{CodeVerifier, IterationContext, VerificationLoop, VerificationResult};

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

// ============================================================================
// TEST 11: Verify valid code succeeds
// ============================================================================

#[test]
fn test_verify_valid_code() {
    let verifier = CodeVerifier::new();
    let code = decy_llm::GeneratedCode {
        code: "fn add(a: i32, b: i32) -> i32 { a + b }".to_string(),
        confidence: 0.9,
        reasoning: "Simple addition".to_string(),
        warnings: vec![],
    };

    let result = verifier.verify(&code).unwrap();
    assert!(result.success);
    assert!(result.compiles);
}

// ============================================================================
// TEST 12: Verify empty code fails
// ============================================================================

#[test]
fn test_verify_empty_code_fails() {
    let verifier = CodeVerifier::new();
    let code = decy_llm::GeneratedCode {
        code: "  ".to_string(),
        confidence: 0.5,
        reasoning: "Empty".to_string(),
        warnings: vec![],
    };

    let result = verifier.verify(&code).unwrap();
    assert!(!result.success);
    assert!(!result.compiles);
    assert!(!result.compile_errors.is_empty());
}

// ============================================================================
// TEST 13: Verify unbalanced braces fails
// ============================================================================

#[test]
fn test_verify_unbalanced_braces_fails() {
    let verifier = CodeVerifier::new();
    let code = decy_llm::GeneratedCode {
        code: "fn foo() { { }".to_string(),
        confidence: 0.5,
        reasoning: "Broken".to_string(),
        warnings: vec![],
    };

    let result = verifier.verify(&code).unwrap();
    assert!(!result.success);
    assert!(!result.compiles);
    assert!(
        result.compile_errors.iter().any(|e| e.contains("braces")),
        "Got: {:?}",
        result.compile_errors
    );
}

// ============================================================================
// TEST 14: Compile empty code returns error
// ============================================================================

#[test]
fn test_compile_empty_code() {
    let verifier = CodeVerifier::new();
    let result = verifier.compile("");
    assert!(result.is_err());
    assert!(result.unwrap_err()[0].contains("Empty"));
}

// ============================================================================
// TEST 15: Compile unbalanced braces returns error
// ============================================================================

#[test]
fn test_compile_unbalanced_braces() {
    let verifier = CodeVerifier::new();
    let result = verifier.compile("fn foo() { {");
    assert!(result.is_err());
    assert!(result.unwrap_err()[0].contains("braces"));
}

// ============================================================================
// TEST 16: Compile valid code succeeds
// ============================================================================

#[test]
fn test_compile_valid_code() {
    let verifier = CodeVerifier::new();
    let result = verifier.compile("fn foo() {}");
    assert!(result.is_ok());
}

// ============================================================================
// TEST 17: Lint detects unwrap
// ============================================================================

#[test]
fn test_lint_detects_unwrap() {
    let verifier = CodeVerifier::new();
    let result = verifier.lint("let x = some_option.unwrap();").unwrap();
    assert!(result >= 1);
}

// ============================================================================
// TEST 18: Lint detects expect
// ============================================================================

#[test]
fn test_lint_detects_expect() {
    let verifier = CodeVerifier::new();
    let result = verifier
        .lint("let x = some_option.expect(\"failed\");")
        .unwrap();
    assert!(result >= 1);
}

// ============================================================================
// TEST 19: Lint detects panic
// ============================================================================

#[test]
fn test_lint_detects_panic() {
    let verifier = CodeVerifier::new();
    let result = verifier.lint("panic!(\"oops\");").unwrap();
    assert!(result >= 1);
}

// ============================================================================
// TEST 20: Lint clean code returns 0
// ============================================================================

#[test]
fn test_lint_clean_code() {
    let verifier = CodeVerifier::new();
    let result = verifier.lint("fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();
    assert_eq!(result, 0);
}

// ============================================================================
// TEST 21: Run tests with test functions
// ============================================================================

#[test]
fn test_run_tests_with_test_attr() {
    let verifier = CodeVerifier::new();
    let result = verifier.run_tests("#[test] fn test_something() { assert!(true); }");
    assert!(result.is_ok());
}

// ============================================================================
// TEST 22: Run tests without test functions
// ============================================================================

#[test]
fn test_run_tests_without_test_attr() {
    let verifier = CodeVerifier::new();
    let result = verifier.run_tests("fn main() {}");
    assert!(result.is_ok());
}

// ============================================================================
// TEST 23: Format feedback with test failures
// ============================================================================

#[test]
fn test_format_feedback_with_test_failures() {
    let loop_runner = VerificationLoop::new(3);

    let result = VerificationResult::test_failure(vec![
        "test_add FAILED: expected 3, got 4".to_string(),
    ]);

    let feedback = loop_runner.format_feedback(&result);
    assert!(
        feedback.contains("Test Failures"),
        "Got: {}",
        feedback
    );
    assert!(
        feedback.contains("test_add"),
        "Got: {}",
        feedback
    );
}

// ============================================================================
// TEST 24: Format feedback with clippy warnings
// ============================================================================

#[test]
fn test_format_feedback_with_clippy_warnings() {
    let loop_runner = VerificationLoop::new(3);

    let mut result = VerificationResult::success();
    result.clippy_warnings = 5;

    let feedback = loop_runner.format_feedback(&result);
    assert!(
        feedback.contains("Clippy Warnings: 5"),
        "Got: {}",
        feedback
    );
}

// ============================================================================
// TEST 25: VerificationLoop default
// ============================================================================

#[test]
fn test_verification_loop_default() {
    let loop_runner = VerificationLoop::default();
    assert_eq!(loop_runner.max_iterations(), 3);
}

// ============================================================================
// TEST 26: CodeVerifier default
// ============================================================================

#[test]
fn test_code_verifier_default() {
    let _verifier = CodeVerifier::default();
}

// ============================================================================
// TEST 27: IterationContext get_feedback with code and errors
// ============================================================================

#[test]
fn test_iteration_context_feedback_includes_previous_code() {
    let mut ctx = IterationContext::new(5);
    ctx.record_failure(
        "fn broken() { let x: i32 = \"bad\"; }",
        vec!["error[E0308]: mismatched types".to_string()],
    );

    let feedback = ctx.get_feedback();
    assert!(
        feedback.contains("Previous Code"),
        "Got: {}",
        feedback
    );
    assert!(
        feedback.contains("fn broken"),
        "Got: {}",
        feedback
    );
    assert!(
        feedback.contains("Previous Errors"),
        "Got: {}",
        feedback
    );
    assert!(
        feedback.contains("Instructions"),
        "Got: {}",
        feedback
    );
}

// ============================================================================
// TEST 28: IterationContext feedback accumulation across iterations
// ============================================================================

#[test]
fn test_iteration_context_feedback_accumulates() {
    let mut ctx = IterationContext::new(5);
    ctx.record_failure("fn v1() {}", vec!["error 1".to_string()]);
    ctx.record_failure("fn v2() {}", vec!["error 2".to_string()]);

    assert_eq!(ctx.feedback.len(), 2);
    assert!(ctx.feedback[0].contains("Iteration 1"));
    assert!(ctx.feedback[1].contains("Iteration 2"));
}

// ============================================================================
// TEST 29: Lint with multiple issues
// ============================================================================

#[test]
fn test_lint_multiple_issues() {
    let verifier = CodeVerifier::new();
    let code = "fn foo() { x.unwrap(); y.expect(\"msg\"); panic!(\"boom\"); }";
    let result = verifier.lint(code).unwrap();
    assert_eq!(result, 3);
}

// ============================================================================
// TEST 30: Format feedback with empty result (no errors, no warnings)
// ============================================================================

#[test]
fn test_format_feedback_empty_result() {
    let loop_runner = VerificationLoop::new(3);
    let result = VerificationResult::success();

    let feedback = loop_runner.format_feedback(&result);
    // No compilation errors, no test failures, no clippy warnings
    assert!(feedback.is_empty(), "Got: {}", feedback);
}
