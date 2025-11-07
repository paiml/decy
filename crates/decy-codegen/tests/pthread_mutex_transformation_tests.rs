//! Integration tests for pthread_mutex → Mutex<T> transformation (DECY-078).
//!
//! Tests that lock analysis results are used to transform C code with locks
//! to Rust's safe Mutex<T> with RAII guards.
//!
//! Coverage:
//! - Lock-protected data wrapped in Mutex<T>
//! - Lock/unlock calls → .lock().unwrap() with RAII
//! - Multiple locks with different protected data
//! - Nested lock regions
//! - Lock discipline validation

use decy_analyzer::lock_analysis::LockAnalyzer;
use decy_codegen::CodeGenerator;
use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirStruct, HirStructField, HirType};

// ============================================================================
// HELPER: Create lock/unlock HIR statements
// ============================================================================

fn lock_call(lock_name: &str) -> HirStatement {
    HirStatement::Expression(HirExpression::FunctionCall {
        function: "pthread_mutex_lock".to_string(),
        arguments: vec![HirExpression::AddressOf(Box::new(
            HirExpression::Variable(lock_name.to_string()),
        ))],
    })
}

fn unlock_call(lock_name: &str) -> HirStatement {
    HirStatement::Expression(HirExpression::FunctionCall {
        function: "pthread_mutex_unlock".to_string(),
        arguments: vec![HirExpression::AddressOf(Box::new(
            HirExpression::Variable(lock_name.to_string()),
        ))],
    })
}

// ============================================================================
// CONCURRENCY INFRASTRUCTURE: Test pthread pattern detection
// ============================================================================

#[test]
fn test_concurrency_module_detects_pthread_calls() {
    use decy_codegen::concurrency_transform;

    // Function with pthread lock/unlock pattern
    let func = HirFunction::new_with_body(
        "increment".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("lock"),
            HirStatement::Assignment {
                target: "counter".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("counter".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
            unlock_call("lock"),
        ],
    );

    // Verify concurrency module can detect pthread patterns
    assert!(
        concurrency_transform::has_pthread_mutex_calls(&func),
        "Should detect pthread_mutex calls"
    );

    // Verify lock region identification
    let regions = concurrency_transform::identify_lock_regions(&func);
    assert_eq!(regions.len(), 1, "Should identify one lock region");
    assert_eq!(regions[0].0, "lock", "Lock name should be 'lock'");
    assert_eq!(regions[0].1, 0, "Lock starts at index 0");
    assert_eq!(regions[0].2, 2, "Lock ends at index 2");
}

// ============================================================================
// LOCK ANALYSIS INTEGRATION: Use LockAnalyzer to detect patterns
// ============================================================================

#[test]
fn test_lock_analyzer_detects_protected_data() {
    // Function with lock protecting counter variable
    let func = HirFunction::new_with_body(
        "increment".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("counter_lock"),
            HirStatement::Assignment {
                target: "counter".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("counter".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
            unlock_call("counter_lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    // Should identify that counter_lock protects counter
    assert!(
        mapping.is_protected_by("counter", "counter_lock"),
        "Expected counter to be protected by counter_lock"
    );

    // This is the pattern that code generation should transform to Mutex<T>
}

// ============================================================================
// CODE GENERATION: Transform lock patterns to Mutex<T>
// ============================================================================

#[test]
#[ignore] // TODO(DECY-078): Implement Mutex<T> transformation in CodeGenerator
fn test_codegen_transforms_lock_to_mutex() {
    // Create a struct with a lock and protected data
    // Based on lock analysis, this should generate Mutex<T>
    let counter_struct = HirStruct::new(
        "Counter".to_string(),
        vec![
            HirStructField::new("lock".to_string(), HirType::Int), // Placeholder for pthread_mutex_t
            HirStructField::new("value".to_string(), HirType::Int),
        ],
    );

    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_struct(&counter_struct);

    // TODO(DECY-078): After implementation, this should transform to:
    // struct Counter {
    //     value: Mutex<i32>
    // }
    //
    // The lock field disappears and the protected data is wrapped in Mutex

    // For now, this test is ignored and will be implemented in GREEN phase
    assert!(
        rust_code.contains("Mutex<i32>"),
        "Expected Mutex<i32> wrapping protected data, got: {}",
        rust_code
    );
}

#[test]
#[ignore] // TODO(DECY-078): Implement lock/unlock transformation
fn test_codegen_transforms_lock_unlock_to_raii() {
    // Function with lock/unlock pattern
    let func = HirFunction::new_with_body(
        "increment".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "c".to_string(),
            HirType::Pointer(Box::new(HirType::Struct("Counter".to_string()))),
        )],
        vec![
            lock_call("c->lock"),
            HirStatement::Assignment {
                target: "c->value".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("c->value".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
            unlock_call("c->lock"),
        ],
    );

    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_function(&func);

    // TODO(DECY-078): After implementation, this should transform to:
    // fn increment(c: &Counter) {
    //     let mut guard = c.value.lock().unwrap();
    //     *guard += 1;
    //     // RAII - no explicit unlock needed
    // }

    // Should use .lock().unwrap() for RAII guard
    assert!(
        rust_code.contains(".lock().unwrap()"),
        "Expected .lock().unwrap() for RAII guard, got: {}",
        rust_code
    );

    // Should NOT have pthread_mutex_lock/unlock functions
    assert!(
        !rust_code.contains("pthread_mutex_lock"),
        "Should not have pthread_mutex_lock, got: {}",
        rust_code
    );
    assert!(
        !rust_code.contains("pthread_mutex_unlock"),
        "Should not have pthread_mutex_unlock, got: {}",
        rust_code
    );
}

// ============================================================================
// MULTIPLE LOCKS: Different mutexes protecting different data
// ============================================================================

#[test]
fn test_lock_analyzer_identifies_multiple_protected_variables() {
    // Function with two separate lock/unlock pairs protecting different data
    let func = HirFunction::new_with_body(
        "update_stats".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("count_lock"),
            HirStatement::Assignment {
                target: "count".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("count".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
            unlock_call("count_lock"),
            lock_call("sum_lock"),
            HirStatement::Assignment {
                target: "sum".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("sum".to_string())),
                    right: Box::new(HirExpression::IntLiteral(10)),
                },
            },
            unlock_call("sum_lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    // count_lock protects count
    assert!(
        mapping.is_protected_by("count", "count_lock"),
        "Expected count to be protected by count_lock"
    );

    // sum_lock protects sum
    assert!(
        mapping.is_protected_by("sum", "sum_lock"),
        "Expected sum to be protected by sum_lock"
    );

    // Cross-check: count NOT protected by sum_lock
    assert!(
        !mapping.is_protected_by("count", "sum_lock"),
        "count should not be protected by sum_lock"
    );
}

#[test]
#[ignore] // TODO(DECY-078): Implement multi-mutex struct transformation
fn test_codegen_handles_multiple_mutexes() {
    // Struct with two locks protecting different data
    let stats_struct = HirStruct::new(
        "Stats".to_string(),
        vec![
            HirStructField::new("count_lock".to_string(), HirType::Int), // pthread_mutex_t placeholder
            HirStructField::new("count".to_string(), HirType::Int),
            HirStructField::new("sum_lock".to_string(), HirType::Int), // pthread_mutex_t placeholder
            HirStructField::new("sum".to_string(), HirType::Int),
        ],
    );

    let codegen = CodeGenerator::new();
    let rust_code = codegen.generate_struct(&stats_struct);

    // TODO(DECY-078): Should transform to:
    // struct Stats {
    //     count: Mutex<i32>,
    //     sum: Mutex<i32>
    // }

    let mutex_count = rust_code.matches("Mutex<").count();
    assert!(
        mutex_count >= 2,
        "Expected at least 2 Mutex types, found {}, code: {}",
        mutex_count,
        rust_code
    );
}

// ============================================================================
// LOCK DISCIPLINE: Violations should be detected
// ============================================================================

#[test]
fn test_lock_discipline_detects_unmatched_lock() {
    // Function with lock but no unlock
    let func = HirFunction::new_with_body(
        "broken".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("data_lock"),
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::IntLiteral(42),
            },
            // Missing unlock!
        ],
    );

    let analyzer = LockAnalyzer::new();
    let violations = analyzer.check_lock_discipline(&func);

    assert!(
        !violations.is_empty(),
        "Expected lock discipline violation for unmatched lock"
    );
    assert!(
        violations[0].contains("Unmatched lock"),
        "Expected 'Unmatched lock' message, got: {}",
        violations[0]
    );
}

#[test]
fn test_lock_discipline_detects_unlock_without_lock() {
    // Function with unlock but no lock
    let func = HirFunction::new_with_body(
        "broken".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::Assignment {
                target: "data".to_string(),
                value: HirExpression::IntLiteral(42),
            },
            unlock_call("data_lock"), // Unlock without lock!
        ],
    );

    let analyzer = LockAnalyzer::new();
    let violations = analyzer.check_lock_discipline(&func);

    assert!(
        !violations.is_empty(),
        "Expected lock discipline violation for unlock without lock"
    );
    assert!(
        violations[0].contains("Unlock without lock"),
        "Expected 'Unlock without lock' message, got: {}",
        violations[0]
    );
}

// ============================================================================
// SINGLE LOCK, MULTIPLE DATA: One lock protecting multiple variables
// ============================================================================

#[test]
fn test_single_lock_protects_multiple_variables() {
    // Lock protects both x and y
    let func = HirFunction::new_with_body(
        "update".to_string(),
        HirType::Void,
        vec![],
        vec![
            lock_call("point_lock"),
            HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::IntLiteral(10),
            },
            HirStatement::Assignment {
                target: "y".to_string(),
                value: HirExpression::IntLiteral(20),
            },
            unlock_call("point_lock"),
        ],
    );

    let analyzer = LockAnalyzer::new();
    let mapping = analyzer.analyze_lock_data_mapping(&func);

    // Both x and y should be protected by point_lock
    assert!(
        mapping.is_protected_by("x", "point_lock"),
        "Expected x to be protected by point_lock"
    );
    assert!(
        mapping.is_protected_by("y", "point_lock"),
        "Expected y to be protected by point_lock"
    );

    // Verify we can get all protected data
    let protected = mapping.get_protected_data("point_lock");
    assert_eq!(protected.len(), 2, "Expected 2 variables protected");
    assert!(protected.contains(&"x".to_string()));
    assert!(protected.contains(&"y".to_string()));
}
