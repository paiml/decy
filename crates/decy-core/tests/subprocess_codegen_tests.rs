//! Tests for fork/exec to Command API code generation (DECY-093).
//!
//! Verifies that detected fork/exec patterns are transformed to
//! idiomatic Rust `std::process::Command` usage.

use decy_codegen::CodeGenerator;
use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirStatement, HirType};

/// Helper: Create test function
fn create_function(name: &str, body: Vec<HirStatement>) -> HirFunction {
    HirFunction::new_with_body(name.to_string(), HirType::Int, vec![], body)
}

// ============================================================================
// TEST 1: Simple exec generates Command::new
// ============================================================================

#[test]
fn test_exec_generates_command_new() {
    // C: execl("/bin/ls", "ls", NULL);
    // Rust: Command::new("/bin/ls").status()?;
    let func = create_function(
        "run_ls",
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "execl".to_string(),
            arguments: vec![
                HirExpression::StringLiteral("/bin/ls".to_string()),
                HirExpression::StringLiteral("ls".to_string()),
                HirExpression::NullLiteral,
            ],
        })],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        code.contains("Command::new"),
        "Should generate Command::new, got:\n{}",
        code
    );
    assert!(
        code.contains("/bin/ls"),
        "Should include command path:\n{}",
        code
    );
}

// ============================================================================
// TEST 2: Exec with args generates Command::new().arg()
// ============================================================================

#[test]
fn test_exec_with_args_generates_arg_chain() {
    // C: execl("/bin/ls", "ls", "-la", NULL);
    // Rust: Command::new("/bin/ls").arg("-la").status()?;
    let func = create_function(
        "run_ls_la",
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "execl".to_string(),
            arguments: vec![
                HirExpression::StringLiteral("/bin/ls".to_string()),
                HirExpression::StringLiteral("ls".to_string()),
                HirExpression::StringLiteral("-la".to_string()),
                HirExpression::NullLiteral,
            ],
        })],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        code.contains(".arg("),
        "Should generate .arg() for arguments:\n{}",
        code
    );
    assert!(
        code.contains("-la"),
        "Should include -la argument:\n{}",
        code
    );
}

// ============================================================================
// TEST 3: Fork+exec pattern generates spawn
// ============================================================================

#[test]
fn test_fork_exec_generates_spawn() {
    // C: pid = fork(); if (pid == 0) execl(...);
    // Rust: Command::new(...).spawn()?;
    let func = create_function(
        "spawn_process",
        vec![
            HirStatement::VariableDeclaration {
                name: "pid".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::FunctionCall {
                    function: "fork".to_string(),
                    arguments: vec![],
                }),
            },
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(HirExpression::Variable("pid".to_string())),
                    right: Box::new(HirExpression::IntLiteral(0)),
                },
                then_block: vec![HirStatement::Expression(HirExpression::FunctionCall {
                    function: "execl".to_string(),
                    arguments: vec![
                        HirExpression::StringLiteral("/bin/echo".to_string()),
                        HirExpression::StringLiteral("echo".to_string()),
                        HirExpression::StringLiteral("hello".to_string()),
                        HirExpression::NullLiteral,
                    ],
                })],
                else_block: None,
            },
        ],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    // Should NOT contain raw fork() call (but comment is OK)
    // Check it's not calling fork() as a function assignment
    assert!(
        !code.contains("= fork();"),
        "Should NOT generate raw fork() assignment:\n{}",
        code
    );
    assert!(
        code.contains("Command::new"),
        "Should generate Command::new:\n{}",
        code
    );
}

// ============================================================================
// TEST 4: Fork+exec+wait generates .wait()
// ============================================================================

#[test]
fn test_fork_exec_wait_generates_wait() {
    // C: pid = fork(); if (pid == 0) exec(...); else waitpid(...);
    // Rust: let child = Command::new(...).spawn()?; child.wait()?;
    let func = create_function(
        "spawn_and_wait",
        vec![
            HirStatement::VariableDeclaration {
                name: "pid".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::FunctionCall {
                    function: "fork".to_string(),
                    arguments: vec![],
                }),
            },
            HirStatement::If {
                condition: HirExpression::BinaryOp {
                    op: BinaryOperator::Equal,
                    left: Box::new(HirExpression::Variable("pid".to_string())),
                    right: Box::new(HirExpression::IntLiteral(0)),
                },
                then_block: vec![HirStatement::Expression(HirExpression::FunctionCall {
                    function: "execl".to_string(),
                    arguments: vec![
                        HirExpression::StringLiteral("/bin/true".to_string()),
                        HirExpression::StringLiteral("true".to_string()),
                        HirExpression::NullLiteral,
                    ],
                })],
                else_block: Some(vec![HirStatement::Expression(
                    HirExpression::FunctionCall {
                        function: "waitpid".to_string(),
                        arguments: vec![
                            HirExpression::Variable("pid".to_string()),
                            HirExpression::AddressOf(Box::new(HirExpression::Variable(
                                "status".to_string(),
                            ))),
                            HirExpression::IntLiteral(0),
                        ],
                    },
                )]),
            },
        ],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        code.contains(".wait()") || code.contains(".status()"),
        "Should generate .wait() or .status():\n{}",
        code
    );
}

// ============================================================================
// TEST 5: execlp uses PATH lookup
// ============================================================================

#[test]
fn test_execlp_uses_path() {
    // C: execlp("ls", "ls", NULL);
    // Rust: Command::new("ls").status()?;  (PATH lookup)
    let func = create_function(
        "run_from_path",
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "execlp".to_string(),
            arguments: vec![
                HirExpression::StringLiteral("ls".to_string()),
                HirExpression::StringLiteral("ls".to_string()),
                HirExpression::NullLiteral,
            ],
        })],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        code.contains("Command::new(\"ls\")"),
        "Should use simple command name for PATH lookup:\n{}",
        code
    );
}

// ============================================================================
// TEST 6: No subprocess patterns - no Command
// ============================================================================

#[test]
fn test_no_subprocess_no_command() {
    let func = create_function(
        "regular_func",
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(42)))],
    );

    let generator = CodeGenerator::new();
    let code = generator.generate_function(&func);

    assert!(
        !code.contains("Command"),
        "Should NOT generate Command for non-subprocess code:\n{}",
        code
    );
}
