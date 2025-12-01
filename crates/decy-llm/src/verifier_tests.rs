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
