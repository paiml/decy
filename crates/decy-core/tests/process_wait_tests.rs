//! Tests for process wait and status handling (DECY-094).
//!
//! Verifies that C wait patterns are transformed to Rust Command API.

use decy_codegen::CodeGenerator;
use decy_hir::{HirExpression, HirFunction, HirStatement, HirType};

/// Helper: Create test function
fn create_function(name: &str, body: Vec<HirStatement>) -> HirFunction {
    HirFunction::new_with_body(name.to_string(), HirType::Int, vec![], body)
}

// ============================================================================
// TEST 1: WEXITSTATUS extracts exit code
// ============================================================================

#[test]
fn test_wexitstatus_generates_code() {
    // C: int code = WEXITSTATUS(status);
    // Rust: status.code().unwrap_or(-1)
    let func = create_function(
        "get_exit_code",
        vec![HirStatement::VariableDeclaration {
            name: "code".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::FunctionCall {
                function: "WEXITSTATUS".to_string(),
                arguments: vec![HirExpression::Variable("status".to_string())],
            }),
        }],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        code.contains(".code()"),
        "Should generate .code() for WEXITSTATUS:\n{}",
        code
    );
}

// ============================================================================
// TEST 2: WIFEXITED checks success
// ============================================================================

#[test]
fn test_wifexited_generates_success() {
    // C: if (WIFEXITED(status)) { ... }
    // Rust: if status.success() { ... }
    let func = create_function(
        "check_exited",
        vec![HirStatement::If {
            condition: HirExpression::FunctionCall {
                function: "WIFEXITED".to_string(),
                arguments: vec![HirExpression::Variable("status".to_string())],
            },
            then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
            else_block: None,
        }],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        code.contains(".success()"),
        "Should generate .success() for WIFEXITED:\n{}",
        code
    );
}

// ============================================================================
// TEST 3: WIFSIGNALED checks if signaled
// ============================================================================

#[test]
fn test_wifsignaled_generates_signal_check() {
    // C: if (WIFSIGNALED(status)) { ... }
    // Rust: if status.signal().is_some() { ... }
    let func = create_function(
        "check_signaled",
        vec![HirStatement::If {
            condition: HirExpression::FunctionCall {
                function: "WIFSIGNALED".to_string(),
                arguments: vec![HirExpression::Variable("status".to_string())],
            },
            then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
            else_block: None,
        }],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        code.contains(".signal()"),
        "Should generate .signal() for WIFSIGNALED:\n{}",
        code
    );
}

// ============================================================================
// TEST 4: WTERMSIG extracts signal number
// ============================================================================

#[test]
fn test_wtermsig_generates_signal() {
    // C: int sig = WTERMSIG(status);
    // Rust: status.signal().unwrap_or(0)
    let func = create_function(
        "get_signal",
        vec![HirStatement::VariableDeclaration {
            name: "sig".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::FunctionCall {
                function: "WTERMSIG".to_string(),
                arguments: vec![HirExpression::Variable("status".to_string())],
            }),
        }],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        code.contains(".signal()"),
        "Should generate .signal() for WTERMSIG:\n{}",
        code
    );
}

// ============================================================================
// TEST 5: wait() generates .wait()
// ============================================================================

#[test]
fn test_wait_generates_wait_call() {
    // C: wait(&status);
    // Rust: child.wait()?;
    let func = create_function(
        "wait_for_child",
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "wait".to_string(),
            arguments: vec![HirExpression::AddressOf(Box::new(HirExpression::Variable(
                "status".to_string(),
            )))],
        })],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        code.contains(".wait()"),
        "Should generate .wait() call:\n{}",
        code
    );
}

// ============================================================================
// TEST 6: No wait macros - no status methods
// ============================================================================

#[test]
fn test_no_wait_macros_no_status() {
    let func = create_function(
        "regular_func",
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        !code.contains(".code()") && !code.contains(".success()") && !code.contains(".signal()"),
        "Should NOT generate status methods for non-wait code:\n{}",
        code
    );
}
