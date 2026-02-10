//! Tests for lock discipline verification (DECY-079).
//!
//! Comprehensive tests to achieve 95%+ coverage.

#[cfg(test)]
mod tests {
    use crate::lock_verify::{LockDisciplineChecker, LockDisciplineReport};
    use decy_analyzer::lock_analysis::LockAnalyzer;
    use decy_hir::{HirExpression, HirFunction, HirStatement, HirType};

    // ========================================================================
    // Helper function to create lock/unlock statements
    // ========================================================================

    fn lock_stmt(mutex_name: &str) -> HirStatement {
        HirStatement::Expression(HirExpression::FunctionCall {
            function: "pthread_mutex_lock".to_string(),
            arguments: vec![HirExpression::AddressOf(Box::new(HirExpression::Variable(
                mutex_name.to_string(),
            )))],
        })
    }

    fn unlock_stmt(mutex_name: &str) -> HirStatement {
        HirStatement::Expression(HirExpression::FunctionCall {
            function: "pthread_mutex_unlock".to_string(),
            arguments: vec![HirExpression::AddressOf(Box::new(HirExpression::Variable(
                mutex_name.to_string(),
            )))],
        })
    }

    fn assign_stmt(target: &str, value: i32) -> HirStatement {
        HirStatement::Assignment {
            target: target.to_string(),
            value: HirExpression::IntLiteral(value),
        }
    }

    // ========================================================================
    // LockDisciplineReport tests
    // ========================================================================

    #[test]
    fn test_lock_discipline_report_is_clean_all_zero() {
        let report = LockDisciplineReport {
            unprotected_accesses: 0,
            lock_violations: 0,
            deadlock_warnings: 0,
        };
        assert!(report.is_clean());
    }

    #[test]
    fn test_lock_discipline_report_is_clean_unprotected_access() {
        let report = LockDisciplineReport {
            unprotected_accesses: 1,
            lock_violations: 0,
            deadlock_warnings: 0,
        };
        assert!(!report.is_clean());
    }

    #[test]
    fn test_lock_discipline_report_is_clean_lock_violation() {
        let report = LockDisciplineReport {
            unprotected_accesses: 0,
            lock_violations: 1,
            deadlock_warnings: 0,
        };
        assert!(!report.is_clean());
    }

    #[test]
    fn test_lock_discipline_report_is_clean_deadlock_warning() {
        let report = LockDisciplineReport {
            unprotected_accesses: 0,
            lock_violations: 0,
            deadlock_warnings: 1,
        };
        assert!(!report.is_clean());
    }

    #[test]
    fn test_lock_discipline_report_is_clean_multiple_violations() {
        let report = LockDisciplineReport {
            unprotected_accesses: 2,
            lock_violations: 1,
            deadlock_warnings: 3,
        };
        assert!(!report.is_clean());
    }

    // ========================================================================
    // LockDisciplineChecker::new tests
    // ========================================================================

    #[test]
    fn test_lock_discipline_checker_new() {
        let analyzer = LockAnalyzer::new();
        let _checker = LockDisciplineChecker::new(&analyzer);
    }

    // ========================================================================
    // check_unprotected_access tests
    // ========================================================================

    #[test]
    fn test_check_unprotected_access_empty_function() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new("test".to_string(), HirType::Void, vec![]);
        let violations = checker.check_unprotected_access(&func);

        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_unprotected_access_no_locks() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![assign_stmt("x", 42)],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_unprotected_access_with_lock_and_protected_access() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("counter", 1),
                unlock_stmt("mutex"),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_unprotected_access_outside_lock_region() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("counter", 1),
                unlock_stmt("mutex"),
                assign_stmt("counter", 2), // Unprotected!
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(!violations.is_empty());
        assert!(violations[0].contains("counter"));
    }

    // ========================================================================
    // check_deadlock_risk tests
    // ========================================================================

    #[test]
    fn test_check_deadlock_risk_empty_functions() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let warnings = checker.check_deadlock_risk(&[]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_check_deadlock_risk_single_function() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![lock_stmt("mutex_a"), lock_stmt("mutex_b")],
        );

        let warnings = checker.check_deadlock_risk(&[func]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_check_deadlock_risk_consistent_ordering() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func1 = HirFunction::new_with_body(
            "test1".to_string(),
            HirType::Void,
            vec![],
            vec![lock_stmt("mutex_a"), lock_stmt("mutex_b")],
        );

        let func2 = HirFunction::new_with_body(
            "test2".to_string(),
            HirType::Void,
            vec![],
            vec![lock_stmt("mutex_a"), lock_stmt("mutex_b")],
        );

        let warnings = checker.check_deadlock_risk(&[func1, func2]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_check_deadlock_risk_inconsistent_ordering() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        // func1: A then B
        let func1 = HirFunction::new_with_body(
            "test1".to_string(),
            HirType::Void,
            vec![],
            vec![lock_stmt("mutex_a"), lock_stmt("mutex_b")],
        );

        // func2: B then A (REVERSE!)
        let func2 = HirFunction::new_with_body(
            "test2".to_string(),
            HirType::Void,
            vec![],
            vec![lock_stmt("mutex_b"), lock_stmt("mutex_a")],
        );

        let warnings = checker.check_deadlock_risk(&[func1, func2]);
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("deadlock"));
    }

    #[test]
    fn test_check_deadlock_risk_no_locks() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func1 = HirFunction::new("test1".to_string(), HirType::Void, vec![]);
        let func2 = HirFunction::new("test2".to_string(), HirType::Void, vec![]);

        let warnings = checker.check_deadlock_risk(&[func1, func2]);
        assert!(warnings.is_empty());
    }

    // ========================================================================
    // check_all tests
    // ========================================================================

    #[test]
    fn test_check_all_clean_function() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new("test".to_string(), HirType::Void, vec![]);
        let report = checker.check_all(&func);

        assert!(report.is_clean());
    }

    #[test]
    fn test_check_all_with_protected_access() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
            ],
        );

        let report = checker.check_all(&func);
        assert!(report.is_clean());
    }

    // ========================================================================
    // Edge case tests for collect_accessed_vars
    // ========================================================================

    #[test]
    fn test_collect_vars_from_expr_variable_declaration() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                HirStatement::VariableDeclaration {
                    name: "local".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::Variable("shared_data".to_string())),
                },
                unlock_stmt("mutex"),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_collect_vars_from_expr_return_statement() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Int,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("shared", 1),
                unlock_stmt("mutex"),
                HirStatement::Return(Some(HirExpression::Variable("shared".to_string()))),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_collect_vars_from_expr_binary_op() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                HirStatement::Assignment {
                    target: "result".to_string(),
                    value: HirExpression::BinaryOp {
                        left: Box::new(HirExpression::Variable("a".to_string())),
                        op: decy_hir::BinaryOperator::Add,
                        right: Box::new(HirExpression::Variable("b".to_string())),
                    },
                },
                unlock_stmt("mutex"),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_collect_vars_from_expr_unary_op() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                HirStatement::Assignment {
                    target: "neg".to_string(),
                    value: HirExpression::UnaryOp {
                        op: decy_hir::UnaryOperator::Minus,
                        operand: Box::new(HirExpression::Variable("x".to_string())),
                    },
                },
                unlock_stmt("mutex"),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_collect_vars_from_expr_function_call() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                HirStatement::Expression(HirExpression::FunctionCall {
                    function: "process".to_string(),
                    arguments: vec![HirExpression::Variable("shared_data".to_string())],
                }),
                unlock_stmt("mutex"),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_collect_vars_from_expr_address_of() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                HirStatement::Assignment {
                    target: "ptr".to_string(),
                    value: HirExpression::AddressOf(Box::new(HirExpression::Variable(
                        "data".to_string(),
                    ))),
                },
                unlock_stmt("mutex"),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_collect_vars_from_expr_dereference() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                HirStatement::Assignment {
                    target: "val".to_string(),
                    value: HirExpression::Dereference(Box::new(HirExpression::Variable(
                        "ptr".to_string(),
                    ))),
                },
                unlock_stmt("mutex"),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_collect_vars_from_expr_array_index() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                HirStatement::Assignment {
                    target: "elem".to_string(),
                    value: HirExpression::ArrayIndex {
                        array: Box::new(HirExpression::Variable("arr".to_string())),
                        index: Box::new(HirExpression::Variable("idx".to_string())),
                    },
                },
                unlock_stmt("mutex"),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_collect_vars_from_expr_field_access() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                HirStatement::Assignment {
                    target: "val".to_string(),
                    value: HirExpression::FieldAccess {
                        object: Box::new(HirExpression::Variable("obj".to_string())),
                        field: "field".to_string(),
                    },
                },
                unlock_stmt("mutex"),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_collect_vars_from_expr_cast() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                HirStatement::Assignment {
                    target: "casted".to_string(),
                    value: HirExpression::Cast {
                        expr: Box::new(HirExpression::Variable("value".to_string())),
                        target_type: HirType::Int,
                    },
                },
                unlock_stmt("mutex"),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_statement_break() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![HirStatement::Break],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_statement_continue() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![HirStatement::Continue],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_is_inside_any_region_boundary_conditions() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),     // idx 0
                assign_stmt("data", 1), // idx 1 (inside)
                unlock_stmt("mutex"),   // idx 2
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_expression_stmt_inside_lock() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                HirStatement::Expression(HirExpression::Variable("x".to_string())),
                unlock_stmt("mutex"),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_variable_decl_without_initializer() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                HirStatement::VariableDeclaration {
                    name: "x".to_string(),
                    var_type: HirType::Int,
                    initializer: None,
                },
                unlock_stmt("mutex"),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_return_none() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("x", 1),
                unlock_stmt("mutex"),
                HirStatement::Return(None),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        // Return(None) doesn't access any variables
        assert!(violations.is_empty());
    }

    // ========================================================================
    // LockDisciplineReport derived trait coverage
    // ========================================================================

    #[test]
    fn test_lock_discipline_report_debug() {
        let report = LockDisciplineReport {
            unprotected_accesses: 1,
            lock_violations: 2,
            deadlock_warnings: 3,
        };
        let debug_str = format!("{:?}", report);
        assert!(debug_str.contains("unprotected_accesses"));
        assert!(debug_str.contains("lock_violations"));
        assert!(debug_str.contains("deadlock_warnings"));
    }

    #[test]
    fn test_lock_discipline_report_clone() {
        let report = LockDisciplineReport {
            unprotected_accesses: 5,
            lock_violations: 3,
            deadlock_warnings: 1,
        };
        let cloned = report.clone();
        assert_eq!(report, cloned);
    }

    #[test]
    fn test_lock_discipline_report_eq() {
        let a = LockDisciplineReport {
            unprotected_accesses: 1,
            lock_violations: 2,
            deadlock_warnings: 3,
        };
        let b = LockDisciplineReport {
            unprotected_accesses: 1,
            lock_violations: 2,
            deadlock_warnings: 3,
        };
        let c = LockDisciplineReport {
            unprotected_accesses: 0,
            lock_violations: 2,
            deadlock_warnings: 3,
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    // ========================================================================
    // collect_accessed_vars: wildcard arm (statements that yield no vars)
    // ========================================================================

    #[test]
    fn test_collect_vars_if_statement_outside_lock() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        // If statement is in the wildcard arm of collect_accessed_vars
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 42),
                unlock_stmt("mutex"),
                HirStatement::If {
                    condition: HirExpression::IntLiteral(1),
                    then_block: vec![],
                    else_block: None,
                },
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        // If statement goes through wildcard arm, collects no vars
        assert!(violations.is_empty());
    }

    #[test]
    fn test_collect_vars_while_statement_outside_lock() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::While {
                    condition: HirExpression::IntLiteral(0),
                    body: vec![],
                },
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_collect_vars_for_statement_outside_lock() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::For {
                    init: vec![],
                    condition: Some(HirExpression::IntLiteral(0)),
                    increment: vec![],
                    body: vec![],
                },
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_collect_vars_switch_statement_outside_lock() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::Switch {
                    condition: HirExpression::IntLiteral(0),
                    cases: vec![],
                    default_case: None,
                },
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_collect_vars_deref_assignment_outside_lock() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::DerefAssignment {
                    target: HirExpression::Variable("ptr".to_string()),
                    value: HirExpression::IntLiteral(10),
                },
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        // DerefAssignment goes through wildcard arm, collects no vars
        assert!(violations.is_empty());
    }

    #[test]
    fn test_collect_vars_array_index_assignment_outside_lock() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::ArrayIndexAssignment {
                    array: Box::new(HirExpression::Variable("arr".to_string())),
                    index: Box::new(HirExpression::IntLiteral(0)),
                    value: HirExpression::IntLiteral(99),
                },
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_collect_vars_field_assignment_outside_lock() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::FieldAssignment {
                    object: HirExpression::Variable("obj".to_string()),
                    field: "x".to_string(),
                    value: HirExpression::IntLiteral(5),
                },
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_collect_vars_free_statement_outside_lock() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::Free {
                    pointer: HirExpression::Variable("ptr".to_string()),
                },
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_collect_vars_inline_asm_outside_lock() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::InlineAsm {
                    text: "nop".to_string(),
                    translatable: false,
                },
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    // ========================================================================
    // collect_vars_from_expr: wildcard arm (expressions that yield no vars)
    // ========================================================================

    #[test]
    fn test_expr_int_literal_no_vars() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                // Expression statement with a literal -- wildcard in collect_vars_from_expr
                HirStatement::Expression(HirExpression::IntLiteral(42)),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_expr_float_literal_no_vars() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::Expression(HirExpression::FloatLiteral("3.14".to_string())),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_expr_string_literal_no_vars() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::Expression(HirExpression::StringLiteral("hello".to_string())),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_expr_char_literal_no_vars() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::Expression(HirExpression::CharLiteral(b'a' as i8)),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_expr_null_literal_no_vars() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::Expression(HirExpression::NullLiteral),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_expr_sizeof_no_vars() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::Expression(HirExpression::Sizeof {
                    type_name: "int".to_string(),
                }),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_expr_post_increment_no_vars_collected() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        // PostIncrement is in the wildcard arm of collect_vars_from_expr
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::Expression(HirExpression::PostIncrement {
                    operand: Box::new(HirExpression::Variable("i".to_string())),
                }),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        // PostIncrement goes through wildcard, so no vars collected
        assert!(violations.is_empty());
    }

    #[test]
    fn test_expr_pre_increment_no_vars_collected() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::Expression(HirExpression::PreIncrement {
                    operand: Box::new(HirExpression::Variable("i".to_string())),
                }),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_expr_post_decrement_no_vars_collected() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::Expression(HirExpression::PostDecrement {
                    operand: Box::new(HirExpression::Variable("i".to_string())),
                }),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_expr_pre_decrement_no_vars_collected() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::Expression(HirExpression::PreDecrement {
                    operand: Box::new(HirExpression::Variable("i".to_string())),
                }),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_expr_is_not_null_no_vars_collected() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::Expression(HirExpression::IsNotNull(Box::new(
                    HirExpression::Variable("ptr".to_string()),
                ))),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_expr_malloc_no_vars_collected() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::Expression(HirExpression::Malloc {
                    size: Box::new(HirExpression::IntLiteral(64)),
                }),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_expr_calloc_no_vars_collected() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::Expression(HirExpression::Calloc {
                    count: Box::new(HirExpression::IntLiteral(10)),
                    element_type: Box::new(HirType::Int),
                }),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_expr_realloc_no_vars_collected() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::Expression(HirExpression::Realloc {
                    pointer: Box::new(HirExpression::Variable("ptr".to_string())),
                    new_size: Box::new(HirExpression::IntLiteral(128)),
                }),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_expr_string_method_call_no_vars_collected() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::Expression(HirExpression::StringMethodCall {
                    receiver: Box::new(HirExpression::Variable("s".to_string())),
                    method: "len".to_string(),
                    arguments: vec![],
                }),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_expr_pointer_field_access_no_vars_collected() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::Expression(HirExpression::PointerFieldAccess {
                    pointer: Box::new(HirExpression::Variable("ptr".to_string())),
                    field: "x".to_string(),
                }),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_expr_slice_index_no_vars_collected() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::Expression(HirExpression::SliceIndex {
                    slice: Box::new(HirExpression::Variable("arr".to_string())),
                    index: Box::new(HirExpression::IntLiteral(0)),
                    element_type: HirType::Int,
                }),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_expr_compound_literal_no_vars_collected() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                HirStatement::Expression(HirExpression::CompoundLiteral {
                    literal_type: HirType::Int,
                    initializers: vec![HirExpression::IntLiteral(1)],
                }),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    // ========================================================================
    // extract_lock_ordering: edge cases
    // ========================================================================

    #[test]
    fn test_extract_lock_ordering_non_address_of_argument() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        // pthread_mutex_lock called with a variable directly (no AddressOf)
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_lock".to_string(),
                arguments: vec![HirExpression::Variable("mutex_ptr".to_string())],
            })],
        );

        // Should produce no lock ordering because argument is not AddressOf
        let warnings = checker.check_deadlock_risk(&[func]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_extract_lock_ordering_empty_arguments() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        // pthread_mutex_lock called with no arguments
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_lock".to_string(),
                arguments: vec![],
            })],
        );

        let warnings = checker.check_deadlock_risk(&[func]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_extract_lock_ordering_address_of_non_variable() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        // pthread_mutex_lock(&(struct_ptr->mutex)) -- AddressOf wrapping a non-Variable
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_lock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::FieldAccess {
                        object: Box::new(HirExpression::Variable("s".to_string())),
                        field: "mutex".to_string(),
                    },
                ))],
            })],
        );

        // Should produce no lock ordering because inner is not Variable
        let warnings = checker.check_deadlock_risk(&[func]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_extract_lock_ordering_non_lock_function_call() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        // A normal function call, not pthread_mutex_lock
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![HirStatement::Expression(HirExpression::FunctionCall {
                function: "printf".to_string(),
                arguments: vec![HirExpression::StringLiteral("hello".to_string())],
            })],
        );

        let warnings = checker.check_deadlock_risk(&[func]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_extract_lock_ordering_non_expression_statements() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        // Function body has only assignment statements, no lock calls
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                assign_stmt("x", 1),
                assign_stmt("y", 2),
            ],
        );

        let warnings = checker.check_deadlock_risk(&[func]);
        assert!(warnings.is_empty());
    }

    // ========================================================================
    // detect_ordering_conflict: partial overlap and no-conflict paths
    // ========================================================================

    #[test]
    fn test_deadlock_risk_disjoint_lock_sets() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        // func1 acquires A, B; func2 acquires C, D (no overlap)
        let func1 = HirFunction::new_with_body(
            "test1".to_string(),
            HirType::Void,
            vec![],
            vec![lock_stmt("mutex_a"), lock_stmt("mutex_b")],
        );

        let func2 = HirFunction::new_with_body(
            "test2".to_string(),
            HirType::Void,
            vec![],
            vec![lock_stmt("mutex_c"), lock_stmt("mutex_d")],
        );

        let warnings = checker.check_deadlock_risk(&[func1, func2]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_deadlock_risk_partial_overlap_no_conflict() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        // func1: A then B; func2: A then C (only A overlaps, no reverse ordering)
        let func1 = HirFunction::new_with_body(
            "test1".to_string(),
            HirType::Void,
            vec![],
            vec![lock_stmt("mutex_a"), lock_stmt("mutex_b")],
        );

        let func2 = HirFunction::new_with_body(
            "test2".to_string(),
            HirType::Void,
            vec![],
            vec![lock_stmt("mutex_a"), lock_stmt("mutex_c")],
        );

        let warnings = checker.check_deadlock_risk(&[func1, func2]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_deadlock_risk_one_lock_present_other_absent() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        // func1: A then B; func2: only B (A not present in func2)
        let func1 = HirFunction::new_with_body(
            "test1".to_string(),
            HirType::Void,
            vec![],
            vec![lock_stmt("mutex_a"), lock_stmt("mutex_b")],
        );

        let func2 = HirFunction::new_with_body(
            "test2".to_string(),
            HirType::Void,
            vec![],
            vec![lock_stmt("mutex_b")],
        );

        // pos_a_in_2 is None, so no conflict detected
        let warnings = checker.check_deadlock_risk(&[func1, func2]);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_deadlock_risk_three_locks_conflict() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        // func1: A then B then C; func2: C then B then A
        let func1 = HirFunction::new_with_body(
            "test1".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex_a"),
                lock_stmt("mutex_b"),
                lock_stmt("mutex_c"),
            ],
        );

        let func2 = HirFunction::new_with_body(
            "test2".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex_c"),
                lock_stmt("mutex_b"),
                lock_stmt("mutex_a"),
            ],
        );

        let warnings = checker.check_deadlock_risk(&[func1, func2]);
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("deadlock"));
    }

    #[test]
    fn test_deadlock_risk_three_functions() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        // func1: A then B; func2: A then B; func3: B then A
        let func1 = HirFunction::new_with_body(
            "test1".to_string(),
            HirType::Void,
            vec![],
            vec![lock_stmt("mutex_a"), lock_stmt("mutex_b")],
        );

        let func2 = HirFunction::new_with_body(
            "test2".to_string(),
            HirType::Void,
            vec![],
            vec![lock_stmt("mutex_a"), lock_stmt("mutex_b")],
        );

        let func3 = HirFunction::new_with_body(
            "test3".to_string(),
            HirType::Void,
            vec![],
            vec![lock_stmt("mutex_b"), lock_stmt("mutex_a")],
        );

        let warnings = checker.check_deadlock_risk(&[func1, func2, func3]);
        // Both func1 vs func3 and func2 vs func3 should produce warnings
        assert!(warnings.len() >= 1);
    }

    #[test]
    fn test_deadlock_risk_single_lock_in_ordering() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        // Functions with only one lock each -- no pair to compare
        let func1 = HirFunction::new_with_body(
            "test1".to_string(),
            HirType::Void,
            vec![],
            vec![lock_stmt("mutex_a")],
        );

        let func2 = HirFunction::new_with_body(
            "test2".to_string(),
            HirType::Void,
            vec![],
            vec![lock_stmt("mutex_b")],
        );

        let warnings = checker.check_deadlock_risk(&[func1, func2]);
        assert!(warnings.is_empty());
    }

    // ========================================================================
    // check_all: non-clean reports
    // ========================================================================

    #[test]
    fn test_check_all_with_unprotected_access() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                unlock_stmt("mutex"),
                assign_stmt("data", 2), // Unprotected access
            ],
        );

        let report = checker.check_all(&func);
        assert!(!report.is_clean());
        assert!(report.unprotected_accesses > 0);
        assert_eq!(report.deadlock_warnings, 0);
    }

    #[test]
    fn test_check_all_with_lock_violation_missing_unlock() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        // Lock without corresponding unlock
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("data", 1),
                // No unlock!
            ],
        );

        let report = checker.check_all(&func);
        assert!(!report.is_clean());
        assert!(report.lock_violations > 0);
    }

    #[test]
    fn test_check_all_with_unlock_without_lock() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        // Unlock without preceding lock
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                unlock_stmt("mutex"),
                assign_stmt("data", 1),
            ],
        );

        let report = checker.check_all(&func);
        assert!(!report.is_clean());
        assert!(report.lock_violations > 0);
    }

    #[test]
    fn test_check_all_deadlock_warnings_always_zero_for_single_func() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        // Even with multiple locks, check_all always sets deadlock_warnings=0
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex_a"),
                lock_stmt("mutex_b"),
            ],
        );

        let report = checker.check_all(&func);
        assert_eq!(report.deadlock_warnings, 0);
    }

    // ========================================================================
    // Unprotected access: violation message format verification
    // ========================================================================

    #[test]
    fn test_unprotected_access_violation_message_format() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("counter", 1),
                unlock_stmt("mutex"),
                assign_stmt("counter", 99),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("Unprotected access to 'counter'"));
        assert!(violations[0].contains("statement 3"));
        assert!(violations[0].contains("outside locked region"));
    }

    #[test]
    fn test_unprotected_access_multiple_vars() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("x", 1),
                assign_stmt("y", 2),
                unlock_stmt("mutex"),
                // Both x and y accessed outside lock
                HirStatement::Assignment {
                    target: "result".to_string(),
                    value: HirExpression::BinaryOp {
                        left: Box::new(HirExpression::Variable("x".to_string())),
                        op: decy_hir::BinaryOperator::Add,
                        right: Box::new(HirExpression::Variable("y".to_string())),
                    },
                },
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        // Should detect violations for both x and y (and potentially "result" too)
        assert!(violations.len() >= 2);
    }

    #[test]
    fn test_unprotected_access_var_decl_with_protected_init_outside_lock() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("shared", 42),
                unlock_stmt("mutex"),
                // VariableDeclaration with initializer that references protected data
                HirStatement::VariableDeclaration {
                    name: "local".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::Variable("shared".to_string())),
                },
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(!violations.is_empty());
        assert!(violations[0].contains("shared"));
    }

    #[test]
    fn test_unprotected_access_expression_stmt_outside_lock() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("shared", 1),
                unlock_stmt("mutex"),
                // Expression statement accessing protected variable outside lock
                HirStatement::Expression(HirExpression::Variable("shared".to_string())),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(!violations.is_empty());
        assert!(violations[0].contains("shared"));
    }

    // ========================================================================
    // Deadlock risk: warning message format verification
    // ========================================================================

    #[test]
    fn test_deadlock_warning_message_format() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func1 = HirFunction::new_with_body(
            "test1".to_string(),
            HirType::Void,
            vec![],
            vec![lock_stmt("alpha"), lock_stmt("beta")],
        );

        let func2 = HirFunction::new_with_body(
            "test2".to_string(),
            HirType::Void,
            vec![],
            vec![lock_stmt("beta"), lock_stmt("alpha")],
        );

        let warnings = checker.check_deadlock_risk(&[func1, func2]);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Potential deadlock"));
        assert!(warnings[0].contains("Inconsistent lock ordering"));
        assert!(warnings[0].contains("alpha"));
        assert!(warnings[0].contains("beta"));
    }

    // ========================================================================
    // Boundary conditions for is_inside_any_region
    // ========================================================================

    #[test]
    fn test_boundary_at_lock_statement_itself() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        // The lock statement (idx 0) and unlock statement (idx 2) are
        // AT the boundaries, so is_inside_any_region uses strict inequality (> and <).
        // lock at idx 0 (start_index), data at idx 1 (inside), unlock at idx 2 (end_index)
        // idx == start_index (0) -> not inside (strict >)
        // idx == end_index (2) -> not inside (strict <)
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),     // idx 0 = start_index
                assign_stmt("data", 1), // idx 1 (inside)
                unlock_stmt("mutex"),   // idx 2 = end_index
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        // Only idx 1 is considered inside the region
        assert!(violations.is_empty());
    }

    #[test]
    fn test_multiple_lock_regions() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),         // idx 0
                assign_stmt("data", 1),     // idx 1 (inside first region)
                unlock_stmt("mutex"),       // idx 2
                lock_stmt("mutex"),         // idx 3
                assign_stmt("data", 2),     // idx 4 (inside second region)
                unlock_stmt("mutex"),       // idx 5
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_access_between_two_lock_regions() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),         // idx 0
                assign_stmt("data", 1),     // idx 1 (inside)
                unlock_stmt("mutex"),       // idx 2
                assign_stmt("data", 99),    // idx 3 (OUTSIDE - between regions)
                lock_stmt("mutex"),         // idx 4
                assign_stmt("data", 2),     // idx 5 (inside)
                unlock_stmt("mutex"),       // idx 6
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(!violations.is_empty());
        // The violation should be about "data" at statement 3
        let has_data_violation = violations.iter().any(|v| v.contains("data") && v.contains("statement 3"));
        assert!(has_data_violation, "Expected violation for 'data' at statement 3, got: {:?}", violations);
    }

    // ========================================================================
    // Nested expression coverage: deep recursion in collect_vars_from_expr
    // ========================================================================

    #[test]
    fn test_deeply_nested_expression() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        // Cast(AddressOf(Dereference(ArrayIndex(Variable, Variable))))
        let deep_expr = HirExpression::Cast {
            target_type: HirType::Int,
            expr: Box::new(HirExpression::AddressOf(Box::new(
                HirExpression::Dereference(Box::new(HirExpression::ArrayIndex {
                    array: Box::new(HirExpression::Variable("arr".to_string())),
                    index: Box::new(HirExpression::Variable("idx".to_string())),
                })),
            ))),
        };

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                HirStatement::Assignment {
                    target: "result".to_string(),
                    value: deep_expr,
                },
                unlock_stmt("mutex"),
                // Access arr outside lock
                HirStatement::Return(Some(HirExpression::Variable("arr".to_string()))),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        let has_arr = violations.iter().any(|v| v.contains("arr"));
        assert!(has_arr, "Expected violation for 'arr', got: {:?}", violations);
    }

    #[test]
    fn test_function_call_with_multiple_args_outside_lock() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                assign_stmt("a", 1),
                assign_stmt("b", 2),
                unlock_stmt("mutex"),
                // Function call with multiple protected args outside lock
                HirStatement::Expression(HirExpression::FunctionCall {
                    function: "process".to_string(),
                    arguments: vec![
                        HirExpression::Variable("a".to_string()),
                        HirExpression::Variable("b".to_string()),
                    ],
                }),
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(violations.len() >= 2);
    }

    #[test]
    fn test_field_access_in_assignment_outside_lock() {
        let analyzer = LockAnalyzer::new();
        let checker = LockDisciplineChecker::new(&analyzer);

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![
                lock_stmt("mutex"),
                HirStatement::Assignment {
                    target: "obj".to_string(),
                    value: HirExpression::IntLiteral(1),
                },
                unlock_stmt("mutex"),
                // FieldAccess on protected object outside lock
                HirStatement::Assignment {
                    target: "val".to_string(),
                    value: HirExpression::FieldAccess {
                        object: Box::new(HirExpression::Variable("obj".to_string())),
                        field: "x".to_string(),
                    },
                },
            ],
        );

        let violations = checker.check_unprotected_access(&func);
        assert!(!violations.is_empty());
    }
}
