//! RED phase tests for lock-to-data binding analysis (DECY-077).

use decy_analyzer::lock_analysis::LockAnalyzer;
use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirStatement, HirType};

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
// SIMPLE LOCK/UNLOCK PATTERN DETECTION
// ============================================================================

#[test]
fn test_detect_simple_lock_unlock_pair() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("data".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let regions = analyzer.find_lock_regions(&func);

    assert_eq!(regions.len(), 1, "Should find one lock/unlock pair");
    assert_eq!(regions[0].lock_name, "lock");
    assert_eq!(regions[0].start_index, 0);
    assert_eq!(regions[0].end_index, 2);
}

#[test]
fn test_identify_data_accessed_in_locked_region() {
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
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    assert!(
        mapping.is_protected_by("data", "lock"),
        "data should be protected by lock"
    );

    let protected = mapping.get_protected_data("lock");
    assert_eq!(protected.len(), 1);
    assert!(protected.contains(&"data".to_string()));
}

// ============================================================================
// MULTIPLE LOCKS AND DATA
// ============================================================================

#[test]
fn test_multiple_locks_protect_different_data() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock1"),
            HirStatement::Assignment {
                target: "data1".to_string(),
                value: HirExpression::IntLiteral(1),
            },
            unlock_call("lock1"),
            lock_call("lock2"),
            HirStatement::Assignment {
                target: "data2".to_string(),
                value: HirExpression::IntLiteral(2),
            },
            unlock_call("lock2"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    assert!(mapping.is_protected_by("data1", "lock1"));
    assert!(!mapping.is_protected_by("data2", "lock1"));
    assert!(mapping.is_protected_by("data2", "lock2"));
    assert!(!mapping.is_protected_by("data1", "lock2"));
}

#[test]
fn test_single_lock_protects_multiple_data() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "data1".to_string(),
                value: HirExpression::IntLiteral(1),
            },
            HirStatement::Assignment {
                target: "data2".to_string(),
                value: HirExpression::IntLiteral(2),
            },
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    let protected = mapping.get_protected_data("lock");
    assert_eq!(protected.len(), 2);
    assert!(protected.contains(&"data1".to_string()));
    assert!(protected.contains(&"data2".to_string()));
}

// ============================================================================
// NESTED LOCKS
// ============================================================================

#[test]
fn test_nested_locks() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("outer_lock"),
            HirStatement::Assignment {
                target: "outer_data".to_string(),
                value: HirExpression::IntLiteral(1),
            },
            lock_call("inner_lock"),
            HirStatement::Assignment {
                target: "inner_data".to_string(),
                value: HirExpression::IntLiteral(2),
            },
            unlock_call("inner_lock"),
            unlock_call("outer_lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let regions = analyzer.find_lock_regions(&func);

    assert_eq!(regions.len(), 2, "Should find two lock regions");

    let outer = regions.iter().find(|r| r.lock_name == "outer_lock");
    assert!(outer.is_some());
    let outer = outer.unwrap();
    assert_eq!(outer.start_index, 0);
    assert_eq!(outer.end_index, 5);

    let inner = regions.iter().find(|r| r.lock_name == "inner_lock");
    assert!(inner.is_some());
    let inner = inner.unwrap();
    assert_eq!(inner.start_index, 2);
    assert_eq!(inner.end_index, 4);
}

// ============================================================================
// LOCK DISCIPLINE VIOLATIONS
// ============================================================================

#[test]
fn test_detect_unmatched_lock() {
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
        ],
    );

    let analyzer = LockAnalyzer::new();
    let violations = analyzer.check_lock_discipline(&func);

    assert_eq!(violations.len(), 1);
    assert!(
        violations[0].contains("unmatched") || violations[0].contains("Unmatched"),
        "{}",
        violations[0]
    );
}

#[test]
fn test_detect_unlock_without_lock() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![unlock_call("lock")],
    );

    let analyzer = LockAnalyzer::new();
    let violations = analyzer.check_lock_discipline(&func);

    assert_eq!(violations.len(), 1);
    assert!(
        violations[0].contains("without lock") || violations[0].contains("Without lock"),
        "{}",
        violations[0]
    );
}

// ============================================================================
// DATA ACCESS PATTERNS
// ============================================================================

#[test]
fn test_data_read_in_locked_region() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::Variable("data".to_string())),
            },
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    assert!(
        mapping.is_protected_by("data", "lock"),
        "Read-only access should count as protected"
    );
}

#[test]
fn test_ignore_local_variables_in_locked_region() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::VariableDeclaration {
                name: "temp".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(42)),
            },
            HirStatement::Assignment {
                target: "global_data".to_string(),
                value: HirExpression::Variable("temp".to_string()),
            },
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    assert!(mapping.is_protected_by("global_data", "lock"));

    let protected = mapping.get_protected_data("lock");
    assert!(protected.contains(&"global_data".to_string()));
}

// ============================================================================
// STATEMENT TYPE COVERAGE TESTS (DECY-COVERAGE)
// ============================================================================

#[test]
fn test_return_with_value_in_locked_region() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Int,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Return(Some(HirExpression::Variable("data".to_string()))),
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    assert!(
        mapping.is_protected_by("data", "lock"),
        "Variable returned should be tracked as protected"
    );
}

#[test]
fn test_return_without_value_in_locked_region() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Return(None),
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    // Empty return should not add any variables
    assert!(mapping.get_protected_data("lock").is_empty());
}

#[test]
fn test_if_statement_in_locked_region() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::If {
                condition: HirExpression::Variable("cond".to_string()),
                then_block: vec![HirStatement::Assignment {
                    target: "then_data".to_string(),
                    value: HirExpression::IntLiteral(1),
                }],
                else_block: Some(vec![HirStatement::Assignment {
                    target: "else_data".to_string(),
                    value: HirExpression::IntLiteral(2),
                }]),
            },
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    assert!(mapping.is_protected_by("cond", "lock"));
    assert!(mapping.is_protected_by("then_data", "lock"));
    assert!(mapping.is_protected_by("else_data", "lock"));
}

#[test]
fn test_while_loop_in_locked_region() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::While {
                condition: HirExpression::Variable("cond".to_string()),
                body: vec![HirStatement::Assignment {
                    target: "loop_data".to_string(),
                    value: HirExpression::Variable("source".to_string()),
                }],
            },
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    assert!(mapping.is_protected_by("cond", "lock"));
    assert!(mapping.is_protected_by("loop_data", "lock"));
    assert!(mapping.is_protected_by("source", "lock"));
}

#[test]
fn test_deref_assignment_in_locked_region() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("ptr".to_string()),
                value: HirExpression::Variable("value".to_string()),
            },
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    assert!(mapping.is_protected_by("ptr", "lock"));
    assert!(mapping.is_protected_by("value", "lock"));
}

#[test]
fn test_array_index_assignment_in_locked_region() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::ArrayIndexAssignment {
                array: Box::new(HirExpression::Variable("arr".to_string())),
                index: Box::new(HirExpression::Variable("idx".to_string())),
                value: HirExpression::Variable("val".to_string()),
            },
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    assert!(mapping.is_protected_by("arr", "lock"));
    assert!(mapping.is_protected_by("idx", "lock"));
    assert!(mapping.is_protected_by("val", "lock"));
}

#[test]
fn test_field_assignment_in_locked_region() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::FieldAssignment {
                object: HirExpression::Variable("obj".to_string()),
                field: "field".to_string(),
                value: HirExpression::Variable("new_value".to_string()),
            },
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    assert!(mapping.is_protected_by("obj", "lock"));
    assert!(mapping.is_protected_by("new_value", "lock"));
}

#[test]
fn test_break_continue_in_locked_region() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Break,
            HirStatement::Continue,
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    // Break and Continue don't access variables
    assert!(mapping.get_protected_data("lock").is_empty());
}

#[test]
fn test_variable_declaration_without_initializer() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::VariableDeclaration {
                name: "x".to_string(),
                var_type: HirType::Int,
                initializer: None,
            },
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    // No initializer means nothing to track
    assert!(mapping.get_protected_data("lock").is_empty());
}

// ============================================================================
// EXPRESSION TYPE COVERAGE TESTS (DECY-COVERAGE)
// ============================================================================

#[test]
fn test_unary_op_expression_in_locked_region() {
    use decy_hir::UnaryOperator;

    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "result".to_string(),
                value: HirExpression::UnaryOp {
                    op: UnaryOperator::Minus,
                    operand: Box::new(HirExpression::Variable("val".to_string())),
                },
            },
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    assert!(mapping.is_protected_by("val", "lock"));
    assert!(mapping.is_protected_by("result", "lock"));
}

#[test]
fn test_dereference_expression_in_locked_region() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "result".to_string(),
                value: HirExpression::Dereference(Box::new(HirExpression::Variable(
                    "ptr".to_string(),
                ))),
            },
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    assert!(mapping.is_protected_by("ptr", "lock"));
}

#[test]
fn test_array_index_expression_in_locked_region() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "result".to_string(),
                value: HirExpression::ArrayIndex {
                    array: Box::new(HirExpression::Variable("arr".to_string())),
                    index: Box::new(HirExpression::Variable("idx".to_string())),
                },
            },
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    assert!(mapping.is_protected_by("arr", "lock"));
    assert!(mapping.is_protected_by("idx", "lock"));
}

#[test]
fn test_field_access_expression_in_locked_region() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "result".to_string(),
                value: HirExpression::FieldAccess {
                    object: Box::new(HirExpression::Variable("obj".to_string())),
                    field: "field".to_string(),
                },
            },
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    assert!(mapping.is_protected_by("obj", "lock"));
}

#[test]
fn test_cast_expression_in_locked_region() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "result".to_string(),
                value: HirExpression::Cast {
                    expr: Box::new(HirExpression::Variable("val".to_string())),
                    target_type: HirType::Float,
                },
            },
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    assert!(mapping.is_protected_by("val", "lock"));
}

#[test]
fn test_literal_expressions_in_locked_region() {
    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "result".to_string(),
                value: HirExpression::IntLiteral(42),
            },
            unlock_call("lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    // Literals don't reference variables, only the assignment target
    let protected = mapping.get_protected_data("lock");
    assert_eq!(protected.len(), 1);
    assert!(protected.contains(&"result".to_string()));
}

// ============================================================================
// INTEGRATION TEST
// ============================================================================

#[test]
fn test_end_to_end_lock_data_mapping() {
    let func = HirFunction::new_with_body(
        "complex".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock1"),
            HirStatement::Assignment {
                target: "data1".to_string(),
                value: HirExpression::IntLiteral(1),
            },
            unlock_call("lock1"),
            lock_call("lock2"),
            HirStatement::Assignment {
                target: "data2".to_string(),
                value: HirExpression::IntLiteral(2),
            },
            HirStatement::Assignment {
                target: "data3".to_string(),
                value: HirExpression::IntLiteral(3),
            },
            unlock_call("lock2"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    assert_eq!(mapping.get_locks().len(), 2);

    let lock1_data = mapping.get_protected_data("lock1");
    assert_eq!(lock1_data.len(), 1);
    assert!(lock1_data.contains(&"data1".to_string()));

    let lock2_data = mapping.get_protected_data("lock2");
    assert_eq!(lock2_data.len(), 2);
    assert!(lock2_data.contains(&"data2".to_string()));
    assert!(lock2_data.contains(&"data3".to_string()));
}
