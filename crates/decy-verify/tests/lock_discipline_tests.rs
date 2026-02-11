//! RED phase tests for lock discipline validation (DECY-079).
//!
//! Tests comprehensive lock discipline checking including:
//! - Unprotected data access detection
//! - Deadlock detection
//! - Lock discipline violation reporting

use decy_analyzer::lock_analysis::LockAnalyzer;
use decy_hir::{HirExpression, HirFunction, HirStatement, HirType, UnaryOperator};
use decy_verify::lock_verify::LockDisciplineChecker;

/// Helper: Create pthread_mutex_lock call
fn lock_call(lock_name: &str) -> HirStatement {
    HirStatement::Expression(HirExpression::FunctionCall {
        function: "pthread_mutex_lock".to_string(),
        arguments: vec![HirExpression::AddressOf(Box::new(HirExpression::Variable(
            lock_name.to_string(),
        )))],
    })
}

/// Helper: Create pthread_mutex_unlock call
fn unlock_call(lock_name: &str) -> HirStatement {
    HirStatement::Expression(HirExpression::FunctionCall {
        function: "pthread_mutex_unlock".to_string(),
        arguments: vec![HirExpression::AddressOf(Box::new(HirExpression::Variable(
            lock_name.to_string(),
        )))],
    })
}

// ============================================================================
// UNPROTECTED DATA ACCESS DETECTION
// ============================================================================

#[test]
fn test_detect_unprotected_data_access() {
    // C code pattern:
    // pthread_mutex_lock(&lock);
    // data = 42;
    // pthread_mutex_unlock(&lock);
    // data = 100;  // VIOLATION: accessing data without lock!

    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::IntLiteral(42),
            },
            unlock_call("lock"),
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::IntLiteral(100),
            },
        ],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let violations = checker.check_unprotected_access(&func);

    assert_eq!(violations.len(), 1, "Should detect one unprotected access");
    assert!(
        violations[0].contains("data"),
        "Violation should mention 'data'"
    );
    assert!(
        violations[0].contains("3") || violations[0].contains("statement 3"),
        "Should include line/statement number: {}",
        violations[0]
    );
}

#[test]
fn test_no_violation_when_data_always_protected() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::IntLiteral(42),
            },
            unlock_call("lock"),
            lock_call("lock"),
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::IntLiteral(100),
            },
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let violations = checker.check_unprotected_access(&func);

    assert_eq!(
        violations.len(),
        0,
        "Should have no violations when all accesses are protected"
    );
}

#[test]
fn test_detect_read_access_without_lock() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::IntLiteral(42),
            },
            unlock_call("lock"),
            // Reading data without lock
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::Variable("data".to_string())),
            },
        ],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let violations = checker.check_unprotected_access(&func);

    assert_eq!(violations.len(), 1, "Should detect unprotected read access");
}

#[test]
fn test_allow_local_variable_without_lock() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "shared_data".to_string(),
                value: HirExpression::IntLiteral(42),
            },
            unlock_call("lock"),
            // Accessing local variable is OK
            HirStatement::VariableDeclaration {
                name: "local".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(100)),
            },
            HirStatement::Assignment {
                target: "local".to_string(),
                value: HirExpression::IntLiteral(200),
            },
        ],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let violations = checker.check_unprotected_access(&func);

    assert_eq!(
        violations.len(),
        0,
        "Local variables should not trigger violations"
    );
}

// ============================================================================
// DEADLOCK DETECTION
// ============================================================================

#[test]
fn test_detect_potential_deadlock_from_lock_ordering() {
    // Thread 1: lock(A) -> lock(B)
    // Thread 2: lock(B) -> lock(A)  <-- potential deadlock!
    // We simulate this by checking inconsistent lock ordering

    let func1 = HirFunction::new_with_body(
        "thread1".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lockA"),
            lock_call("lockB"),
            unlock_call("lockB"),
            unlock_call("lockA"),
        ],
    );

    let func2 = HirFunction::new_with_body(
        "thread2".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lockB"),
            lock_call("lockA"),
            unlock_call("lockA"),
            unlock_call("lockB"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let warnings = checker.check_deadlock_risk(&[func1, func2]);

    assert!(
        !warnings.is_empty(),
        "Should detect potential deadlock from inconsistent lock ordering"
    );
    assert!(
        warnings[0].contains("deadlock") || warnings[0].contains("Deadlock"),
        "Warning should mention deadlock: {}",
        warnings[0]
    );
    assert!(
        warnings[0].contains("lockA") && warnings[0].contains("lockB"),
        "Should mention both locks: {}",
        warnings[0]
    );
}

#[test]
fn test_no_deadlock_warning_for_consistent_ordering() {
    // Both functions acquire locks in same order: A then B
    let func1 = HirFunction::new_with_body(
        "thread1".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lockA"),
            lock_call("lockB"),
            unlock_call("lockB"),
            unlock_call("lockA"),
        ],
    );

    let func2 = HirFunction::new_with_body(
        "thread2".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lockA"),
            lock_call("lockB"),
            unlock_call("lockB"),
            unlock_call("lockA"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let warnings = checker.check_deadlock_risk(&[func1, func2]);

    assert_eq!(
        warnings.len(),
        0,
        "Consistent lock ordering should not trigger deadlock warnings"
    );
}

#[test]
fn test_single_lock_no_deadlock() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![lock_call("lock"), unlock_call("lock")],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let warnings = checker.check_deadlock_risk(&[func]);

    assert_eq!(warnings.len(), 0, "Single lock cannot cause deadlock");
}

// ============================================================================
// COMPREHENSIVE DISCIPLINE CHECKING
// ============================================================================

#[test]
fn test_comprehensive_discipline_check() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::IntLiteral(42),
            },
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let report = checker.check_all(&func);

    assert_eq!(report.unprotected_accesses, 0);
    assert_eq!(report.lock_violations, 0);
    assert_eq!(report.deadlock_warnings, 0);
    assert!(report.is_clean(), "Should be clean with proper lock usage");
}

#[test]
fn test_report_all_violation_types() {
    // Function with:
    // 1. Unmatched lock
    // 2. Unprotected access
    let func = HirFunction::new_with_body(
        "bad".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::IntLiteral(42),
            },
            unlock_call("lock"),
            // Unprotected access
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::IntLiteral(100),
            },
            // Unmatched lock
            lock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let report = checker.check_all(&func);

    assert!(
        report.unprotected_accesses > 0,
        "Should detect unprotected access"
    );
    assert!(report.lock_violations > 0, "Should detect unmatched lock");
    assert!(!report.is_clean(), "Should not be clean with violations");
}

// ============================================================================
// EDGE CASES: EMPTY, BREAK, CONTINUE, RETURN(NONE)
// ============================================================================

#[test]
fn test_deadlock_risk_empty_functions_list() {
    // Edge case: empty slice → early return (line 253-254)
    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let warnings = checker.check_deadlock_risk(&[]);
    assert!(warnings.is_empty(), "Empty functions list cannot deadlock");
}

#[test]
fn test_collect_accessed_vars_default_arm() {
    // Break, Continue, Return(None) outside lock region trigger default arm (line 202)
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::IntLiteral(42),
            },
            unlock_call("lock"),
            HirStatement::Break,
            HirStatement::Continue,
            HirStatement::Return(None),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let violations = checker.check_unprotected_access(&func);
    assert!(
        violations.is_empty(),
        "Break/Continue/Return(None) don't access data"
    );
}

// ============================================================================
// EXPRESSION VARIANTS IN collect_vars_from_expr
// ============================================================================

#[test]
fn test_unprotected_access_through_unary_op() {
    // UnaryOp accessing protected data outside lock (lines 220-221)
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::IntLiteral(42),
            },
            unlock_call("lock"),
            HirStatement::Expression(HirExpression::UnaryOp {
                op: UnaryOperator::Minus,
                operand: Box::new(HirExpression::Variable("data".to_string())),
            }),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let violations = checker.check_unprotected_access(&func);
    assert!(
        !violations.is_empty(),
        "Should detect unprotected access through UnaryOp"
    );
}

#[test]
fn test_unprotected_access_through_function_call_args() {
    // FunctionCall with protected data in args (lines 224-226)
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::IntLiteral(42),
            },
            unlock_call("lock"),
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "printf".to_string(),
                arguments: vec![HirExpression::Variable("data".to_string())],
            }),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let violations = checker.check_unprotected_access(&func);
    assert!(
        !violations.is_empty(),
        "Should detect unprotected access through function call args"
    );
}

#[test]
fn test_unprotected_access_through_cast() {
    // Cast expression accessing protected data (lines 238-240)
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::IntLiteral(42),
            },
            unlock_call("lock"),
            HirStatement::Expression(HirExpression::Cast {
                expr: Box::new(HirExpression::Variable("data".to_string())),
                target_type: HirType::Float,
            }),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let violations = checker.check_unprotected_access(&func);
    assert!(
        !violations.is_empty(),
        "Should detect unprotected access through cast"
    );
}

#[test]
fn test_unprotected_return_with_protected_data() {
    // Return(Some(expr)) accessing protected data (lines 199-200)
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::IntLiteral(42),
            },
            unlock_call("lock"),
            HirStatement::Return(Some(HirExpression::Variable("data".to_string()))),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let violations = checker.check_unprotected_access(&func);
    assert!(
        !violations.is_empty(),
        "Should detect unprotected access in return"
    );
}

// ============================================================================
// LOCK ORDERING EDGE CASES
// ============================================================================

#[test]
fn test_lock_with_non_addressof_arg_not_tracked() {
    // pthread_mutex_lock(lock) instead of pthread_mutex_lock(&lock)
    // The ordering extractor should skip non-AddressOf args
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "pthread_mutex_lock".to_string(),
            arguments: vec![HirExpression::Variable("lock".to_string())],
        })],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let warnings = checker.check_deadlock_risk(&[func]);
    assert!(
        warnings.is_empty(),
        "Non-AddressOf lock args shouldn't produce ordering"
    );
}

#[test]
fn test_three_lock_ordering_conflict() {
    // func1: lock(A) → lock(B) → lock(C)
    // func2: lock(C) → lock(A) → lock(B)
    // Conflict: A before C in func1, C before A in func2
    let func1 = HirFunction::new_with_body(
        "thread1".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lockA"),
            lock_call("lockB"),
            lock_call("lockC"),
            unlock_call("lockC"),
            unlock_call("lockB"),
            unlock_call("lockA"),
        ],
    );

    let func2 = HirFunction::new_with_body(
        "thread2".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lockC"),
            lock_call("lockA"),
            lock_call("lockB"),
            unlock_call("lockB"),
            unlock_call("lockA"),
            unlock_call("lockC"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let warnings = checker.check_deadlock_risk(&[func1, func2]);
    assert!(
        !warnings.is_empty(),
        "Should detect deadlock with 3-lock ordering conflict"
    );
}

#[test]
fn test_unprotected_access_through_array_index() {
    // ArrayIndex accessing protected data outside lock (lines 231-234)
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::IntLiteral(42),
            },
            unlock_call("lock"),
            // Accessing data[0] outside lock
            HirStatement::Expression(HirExpression::ArrayIndex {
                array: Box::new(HirExpression::Variable("data".to_string())),
                index: Box::new(HirExpression::Variable("i".to_string())),
            }),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let violations = checker.check_unprotected_access(&func);
    assert!(
        !violations.is_empty(),
        "Should detect unprotected access through ArrayIndex"
    );
    assert!(
        violations[0].contains("data"),
        "Violation should mention 'data': {}",
        violations[0]
    );
}

#[test]
fn test_unprotected_access_through_addressof_dereference() {
    // AddressOf and Dereference accessing protected data outside lock (lines 228-230)
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::IntLiteral(42),
            },
            unlock_call("lock"),
            HirStatement::Expression(HirExpression::AddressOf(Box::new(
                HirExpression::Variable("data".to_string()),
            ))),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let violations = checker.check_unprotected_access(&func);
    assert!(
        !violations.is_empty(),
        "Should detect unprotected access through AddressOf"
    );
}

#[test]
fn test_unprotected_access_through_field_access() {
    // FieldAccess accessing protected data outside lock (lines 235-237)
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "obj".to_string(),
                value: HirExpression::IntLiteral(42),
            },
            unlock_call("lock"),
            HirStatement::Expression(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("obj".to_string())),
                field: "x".to_string(),
            }),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let violations = checker.check_unprotected_access(&func);
    assert!(
        !violations.is_empty(),
        "Should detect unprotected access through FieldAccess"
    );
}

#[test]
fn test_unprotected_dereference_outside_lock() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "ptr".to_string(),
                value: HirExpression::IntLiteral(0),
            },
            unlock_call("lock"),
            HirStatement::Expression(HirExpression::Dereference(Box::new(
                HirExpression::Variable("ptr".to_string()),
            ))),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let violations = checker.check_unprotected_access(&func);
    assert!(
        !violations.is_empty(),
        "Should detect unprotected access through Dereference"
    );
}

#[test]
fn test_unprotected_binary_op_outside_lock() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "counter".to_string(),
                value: HirExpression::IntLiteral(0),
            },
            unlock_call("lock"),
            HirStatement::Expression(HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("counter".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            }),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let violations = checker.check_unprotected_access(&func);
    assert!(
        !violations.is_empty(),
        "Should detect unprotected access through BinaryOp"
    );
}

#[test]
fn test_lock_discipline_report_not_clean_with_all_violations() {
    let report = decy_verify::lock_verify::LockDisciplineReport {
        unprotected_accesses: 2,
        lock_violations: 1,
        deadlock_warnings: 3,
    };
    assert!(!report.is_clean());
}

#[test]
fn test_deadlock_risk_functions_with_no_locks() {
    // Functions that don't use any locks should not produce ordering warnings
    let func1 = HirFunction::new_with_body(
        "func1".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::IntLiteral(1),
        }],
    );
    let func2 = HirFunction::new_with_body(
        "func2".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Assignment {
            target: "y".to_string(),
            value: HirExpression::IntLiteral(2),
        }],
    );

    let analyzer = LockAnalyzer::new();
    let checker = LockDisciplineChecker::new(&analyzer);
    let warnings = checker.check_deadlock_risk(&[func1, func2]);
    assert!(warnings.is_empty(), "No-lock functions cannot deadlock");
}
