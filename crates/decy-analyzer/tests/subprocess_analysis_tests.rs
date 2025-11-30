//! Tests for fork/exec subprocess pattern detection (DECY-092).

use decy_analyzer::subprocess_analysis::{ForkExecPattern, SubprocessDetector};
use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirStatement, HirType};

/// Helper: Create test function
fn create_test_function(name: &str, body: Vec<HirStatement>) -> HirFunction {
    HirFunction::new_with_body(name.to_string(), HirType::Int, vec![], body)
}

// ============================================================================
// TEST 1: Detect simple fork() call
// ============================================================================

#[test]
fn test_detect_fork_call() {
    // C: pid_t pid = fork();
    let func = create_test_function(
        "spawn_process",
        vec![HirStatement::VariableDeclaration {
            name: "pid".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::FunctionCall {
                function: "fork".to_string(),
                arguments: vec![],
            }),
        }],
    );

    let detector = SubprocessDetector::new();
    let patterns = detector.detect(&func);

    assert!(!patterns.is_empty(), "Should detect fork call");
    assert!(patterns[0].has_fork);
}

// ============================================================================
// TEST 2: Detect fork + exec pattern
// ============================================================================

#[test]
fn test_detect_fork_exec_pattern() {
    // C: pid = fork(); if (pid == 0) { execl("/bin/ls", "ls", NULL); }
    let func = create_test_function(
        "run_command",
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
                        HirExpression::StringLiteral("/bin/ls".to_string()),
                        HirExpression::StringLiteral("ls".to_string()),
                        HirExpression::NullLiteral,
                    ],
                })],
                else_block: None,
            },
        ],
    );

    let detector = SubprocessDetector::new();
    let patterns = detector.detect(&func);

    assert_eq!(patterns.len(), 1);
    assert!(patterns[0].has_fork);
    assert!(patterns[0].has_exec);
    assert_eq!(patterns[0].command, Some("/bin/ls".to_string()));
}

// ============================================================================
// TEST 3: Detect execv with argument array
// ============================================================================

#[test]
fn test_detect_execv_with_args() {
    let func = create_test_function(
        "run_with_args",
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "execv".to_string(),
            arguments: vec![
                HirExpression::StringLiteral("/usr/bin/gcc".to_string()),
                HirExpression::Variable("argv".to_string()),
            ],
        })],
    );

    let detector = SubprocessDetector::new();
    let patterns = detector.detect(&func);

    assert!(!patterns.is_empty());
    assert!(patterns[0].has_exec);
    assert_eq!(patterns[0].command, Some("/usr/bin/gcc".to_string()));
}

// ============================================================================
// TEST 4: Detect waitpid in parent branch
// ============================================================================

#[test]
fn test_detect_waitpid_pattern() {
    // C: if (pid > 0) { waitpid(pid, &status, 0); }
    let func = create_test_function(
        "wait_for_child",
        vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::Variable("pid".to_string())),
                right: Box::new(HirExpression::IntLiteral(0)),
            },
            then_block: vec![HirStatement::Expression(HirExpression::FunctionCall {
                function: "waitpid".to_string(),
                arguments: vec![
                    HirExpression::Variable("pid".to_string()),
                    HirExpression::AddressOf(Box::new(HirExpression::Variable(
                        "status".to_string(),
                    ))),
                    HirExpression::IntLiteral(0),
                ],
            })],
            else_block: None,
        }],
    );

    let detector = SubprocessDetector::new();
    let patterns = detector.detect(&func);

    assert!(!patterns.is_empty());
    assert!(patterns[0].has_wait);
}

// ============================================================================
// TEST 5: Complete fork/exec/wait pattern
// ============================================================================

#[test]
fn test_complete_subprocess_pattern() {
    let func = create_test_function(
        "full_subprocess",
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
                    function: "execlp".to_string(),
                    arguments: vec![
                        HirExpression::StringLiteral("ls".to_string()),
                        HirExpression::StringLiteral("ls".to_string()),
                        HirExpression::StringLiteral("-la".to_string()),
                        HirExpression::NullLiteral,
                    ],
                })],
                else_block: Some(vec![HirStatement::Expression(
                    HirExpression::FunctionCall {
                        function: "wait".to_string(),
                        arguments: vec![HirExpression::AddressOf(Box::new(
                            HirExpression::Variable("status".to_string()),
                        ))],
                    },
                )]),
            },
        ],
    );

    let detector = SubprocessDetector::new();
    let patterns = detector.detect(&func);

    assert_eq!(patterns.len(), 1);
    let p = &patterns[0];
    assert!(p.has_fork, "Should have fork");
    assert!(p.has_exec, "Should have exec");
    assert!(p.has_wait, "Should have wait");
    assert_eq!(p.command, Some("ls".to_string()));
    assert_eq!(p.args, vec!["ls", "-la"]);
}

// ============================================================================
// TEST 6: No subprocess pattern (negative test)
// ============================================================================

#[test]
fn test_no_subprocess_pattern() {
    let func = create_test_function(
        "regular_function",
        vec![HirStatement::Return(Some(HirExpression::IntLiteral(42)))],
    );

    let detector = SubprocessDetector::new();
    let patterns = detector.detect(&func);

    assert!(patterns.is_empty(), "Should not detect subprocess pattern");
}
