//! RED phase tests for lock-to-data binding analysis (DECY-077).
//!
//! Tests the ability to analyze C code with pthread locks and determine
//! which locks protect which data variables.
//!
//! Test Coverage:
//! - Simple lock/unlock pairs
//! - Data access within locked regions
//! - Multiple locks protecting different data
//! - Nested locks
//! - Lock discipline violations

use decy_analyzer::lock_analysis::{LockAnalyzer, LockRegion, LockDataMapping};
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

// ============================================================================
// SIMPLE LOCK/UNLOCK PATTERN DETECTION
// ============================================================================

#[test]
fn test_detect_simple_lock_unlock_pair() {
    // C code pattern:
    // pthread_mutex_lock(&lock);
    // data++;
    // pthread_mutex_unlock(&lock);

    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            // pthread_mutex_lock(&lock)
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_lock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("lock".to_string()),
                ))],
            }),
            // data++
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::BinaryOp {
                    op: decy_hir::BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("data".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            }),
            // pthread_mutex_unlock(&lock)
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_unlock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("lock".to_string()),
                ))],
            }),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let regions = analyzer.find_lock_regions(&func);

    // Should find one locked region
    assert_eq!(
        regions.len(),
        1,
        "Should find one lock/unlock pair, found: {:?}",
        regions
    );

    let region = &regions[0];
    assert_eq!(region.lock_name, "lock", "Lock name should be 'lock'");
    assert_eq!(region.start_index, 0, "Lock should start at statement 0");
    assert_eq!(region.end_index, 2, "Lock should end at statement 2");
}

#[test]
fn test_identify_data_accessed_in_locked_region() {
    // Same pattern as above - should identify 'data' as protected by 'lock'

    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_lock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("lock".to_string()),
                ))],
            }),
            HirStatement::Assignment {
                target: "data".to_string(),
                initializer: HirExpression::IntLiteral(42),
            }),
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_unlock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("lock".to_string()),
                ))],
            }),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    // Should map "lock" → ["data"]
    assert!(
        mapping.is_protected_by("data", "lock"),
        "Data 'data' should be protected by lock 'lock'"
    );

    let protected_data = mapping.get_protected_data("lock");
    assert_eq!(
        protected_data.len(),
        1,
        "Lock should protect exactly one variable"
    );
    assert!(
        protected_data.contains(&"data".to_string()),
        "Protected data should include 'data'"
    );
}

// ============================================================================
// MULTIPLE LOCKS AND DATA
// ============================================================================

#[test]
fn test_multiple_locks_protect_different_data() {
    // Pattern:
    // pthread_mutex_lock(&lock1); data1++; pthread_mutex_unlock(&lock1);
    // pthread_mutex_lock(&lock2); data2++; pthread_mutex_unlock(&lock2);

    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            // First lock region
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_lock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("lock1".to_string()),
                ))],
            }),
            HirStatement::Assignment {
                target: "data1".to_string(),
                value: HirExpression::IntLiteral(1),
            }),
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_unlock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("lock1".to_string()),
                ))],
            }),
            // Second lock region
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_lock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("lock2".to_string()),
                ))],
            }),
            HirStatement::Assignment {
                target: "data2".to_string(),
                value: HirExpression::IntLiteral(2),
            }),
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_unlock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("lock2".to_string()),
                ))],
            }),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    // lock1 → data1
    assert!(
        mapping.is_protected_by("data1", "lock1"),
        "data1 should be protected by lock1"
    );
    assert!(
        !mapping.is_protected_by("data2", "lock1"),
        "data2 should NOT be protected by lock1"
    );

    // lock2 → data2
    assert!(
        mapping.is_protected_by("data2", "lock2"),
        "data2 should be protected by lock2"
    );
    assert!(
        !mapping.is_protected_by("data1", "lock2"),
        "data1 should NOT be protected by lock2"
    );
}

#[test]
fn test_single_lock_protects_multiple_data() {
    // Pattern:
    // pthread_mutex_lock(&lock);
    // data1++;
    // data2++;
    // pthread_mutex_unlock(&lock);

    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_lock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("lock".to_string()),
                ))],
            }),
            HirStatement::Assignment {
                target: "data1".to_string(),
                value: HirExpression::IntLiteral(1),
            }),
            HirStatement::Assignment {
                target: "data2".to_string(),
                value: HirExpression::IntLiteral(2),
            }),
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_unlock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("lock".to_string()),
                ))],
            }),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    let protected_data = mapping.get_protected_data("lock");
    assert_eq!(
        protected_data.len(),
        2,
        "Lock should protect two variables"
    );
    assert!(
        protected_data.contains(&"data1".to_string()),
        "Should protect data1"
    );
    assert!(
        protected_data.contains(&"data2".to_string()),
        "Should protect data2"
    );
}

// ============================================================================
// NESTED LOCKS
// ============================================================================

#[test]
fn test_nested_locks() {
    // Pattern:
    // pthread_mutex_lock(&outer_lock);
    //   outer_data++;
    //   pthread_mutex_lock(&inner_lock);
    //     inner_data++;
    //   pthread_mutex_unlock(&inner_lock);
    // pthread_mutex_unlock(&outer_lock);

    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            // Outer lock
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_lock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("outer_lock".to_string()),
                ))],
            }),
            HirStatement::Assignment {
                target: "outer_data".to_string(),
                value: HirExpression::IntLiteral(1),
            }),
            // Inner lock
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_lock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("inner_lock".to_string()),
                ))],
            }),
            HirStatement::Assignment {
                target: "inner_data".to_string(),
                value: HirExpression::IntLiteral(2),
            }),
            // Inner unlock
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_unlock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("inner_lock".to_string()),
                ))],
            }),
            // Outer unlock
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_unlock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("outer_lock".to_string()),
                ))],
            }),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let regions = analyzer.find_lock_regions(&func);

    // Should find two locked regions
    assert_eq!(
        regions.len(),
        2,
        "Should find two lock regions (nested)"
    );

    // Outer region should span indices 0-5
    let outer = regions.iter().find(|r| r.lock_name == "outer_lock");
    assert!(outer.is_some(), "Should find outer_lock region");
    let outer = outer.unwrap();
    assert_eq!(outer.start_index, 0);
    assert_eq!(outer.end_index, 5);

    // Inner region should span indices 2-4
    let inner = regions.iter().find(|r| r.lock_name == "inner_lock");
    assert!(inner.is_some(), "Should find inner_lock region");
    let inner = inner.unwrap();
    assert_eq!(inner.start_index, 2);
    assert_eq!(inner.end_index, 4);
}

// ============================================================================
// LOCK DISCIPLINE VIOLATIONS
// ============================================================================

#[test]
fn test_detect_unmatched_lock() {
    // Pattern: lock without unlock (error)
    // pthread_mutex_lock(&lock);
    // data++;
    // (missing unlock)

    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_lock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("lock".to_string()),
                ))],
            }),
            HirStatement::Assignment {
                target: "data".to_string(),
                initializer: HirExpression::IntLiteral(42),
            }),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let violations = analyzer.check_lock_discipline(&func);

    assert_eq!(
        violations.len(),
        1,
        "Should detect unmatched lock as violation"
    );
    assert!(
        violations[0].contains("unmatched") || violations[0].contains("Unmatched"),
        "Violation should mention unmatched lock: {}",
        violations[0]
    );
}

#[test]
fn test_detect_unlock_without_lock() {
    // Pattern: unlock without lock (error)
    // pthread_mutex_unlock(&lock);

    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::Expression(HirExpression::FunctionCall {
            function: "pthread_mutex_unlock".to_string(),
            arguments: vec![HirExpression::AddressOf(Box::new(
                HirExpression::Variable("lock".to_string()),
            ))],
        }],
    );

    let analyzer = LockAnalyzer::new();
    let violations = analyzer.check_lock_discipline(&func);

    assert_eq!(
        violations.len(),
        1,
        "Should detect unlock without lock as violation"
    );
    assert!(
        violations[0].contains("without lock") || violations[0].contains("Without lock"),
        "Violation should mention unlock without lock: {}",
        violations[0]
    );
}

// ============================================================================
// DATA ACCESS PATTERNS
// ============================================================================

#[test]
fn test_data_read_in_locked_region() {
    // Pattern: read data within lock
    // pthread_mutex_lock(&lock);
    // x = data;
    // pthread_mutex_unlock(&lock);

    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_lock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("lock".to_string()),
                ))],
            }),
            HirStatement::VariableDeclaration {
                function: "x".to_string(),
                var_type: HirType::Int,
                initializer: HirExpression::Variable("data".to_string()),
            }),
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_unlock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("lock".to_string()),
                ))],
            }),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    // Should detect 'data' as protected even when only read
    assert!(
        mapping.is_protected_by("data", "lock"),
        "Read-only access should still count as protected data"
    );
}

#[test]
fn test_ignore_local_variables_in_locked_region() {
    // Pattern: local variable in locked region shouldn't be considered protected
    // pthread_mutex_lock(&lock);
    // int temp = 42;  // local, not protected
    // global_data = temp;  // global_data IS protected
    // pthread_mutex_unlock(&lock);

    let func = HirFunction::new_with_body(
        "test".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_lock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("lock".to_string()),
                ))],
            }),
            HirStatement::VariableDeclaration {
                function: "temp".to_string(),
                var_type: HirType::Int,
                initializer: HirExpression::IntLiteral(42),
            }),
            HirStatement::Assignment {
                target: "global_data".to_string(),
                initializer: HirExpression::Variable("temp".to_string()),
            }),
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_unlock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("lock".to_string()),
                ))],
            }),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    // global_data should be protected
    assert!(
        mapping.is_protected_by("global_data", "lock"),
        "Global data should be protected"
    );

    // temp is local - implementation may or may not track it
    // For now, just verify global_data is tracked
    let protected = mapping.get_protected_data("lock");
    assert!(
        protected.contains(&"global_data".to_string()),
        "Protected data should include global_data"
    );
}

// ============================================================================
// INTEGRATION TEST: FULL LOCK-DATA MAPPING
// ============================================================================

#[test]
fn test_end_to_end_lock_data_mapping() {
    // Complex pattern with multiple locks and data
    let func = HirFunction::new_with_body(
        "complex_function".to_string(),
        HirType::Void,
        vec![],
        vec![
            // lock1 protects data1
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_lock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("lock1".to_string()),
                ))],
            }),
            HirStatement::Assignment {
                target: "data1".to_string(),
                value: HirExpression::IntLiteral(1),
            }),
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_unlock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("lock1".to_string()),
                ))],
            }),
            // lock2 protects data2 and data3
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_lock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("lock2".to_string()),
                ))],
            }),
            HirStatement::Assignment {
                target: "data2".to_string(),
                value: HirExpression::IntLiteral(2),
            }),
            HirStatement::Assignment {
                target: "data3".to_string(),
                value: HirExpression::IntLiteral(3),
            }),
            HirStatement::Expression(HirExpression::FunctionCall {
                function: "pthread_mutex_unlock".to_string(),
                arguments: vec![HirExpression::AddressOf(Box::new(
                    HirExpression::Variable("lock2".to_string()),
                ))],
            }),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    // Verify complete mapping
    assert_eq!(
        mapping.get_locks().len(),
        2,
        "Should track 2 distinct locks"
    );

    let lock1_data = mapping.get_protected_data("lock1");
    assert_eq!(lock1_data.len(), 1, "lock1 should protect 1 variable");
    assert!(lock1_data.contains(&"data1".to_string()));

    let lock2_data = mapping.get_protected_data("lock2");
    assert_eq!(lock2_data.len(), 2, "lock2 should protect 2 variables");
    assert!(lock2_data.contains(&"data2".to_string()));
    assert!(lock2_data.contains(&"data3".to_string()));
}
