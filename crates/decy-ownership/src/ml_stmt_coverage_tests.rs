//! Coverage tests for count_stmt_accesses, count_accesses, count_null_checks_in_stmt,
//! expr_uses_var, is_null_check, and related functions in ml_features.rs.
//!
//! Targets the 42 uncovered lines starting at line 684 (count_stmt_accesses)
//! and nearby helper functions.

use crate::ml_features::*;
use decy_hir::{
    BinaryOperator, HirExpression, HirFunction, HirParameter, HirStatement, HirType,
    UnaryOperator,
};

// ============================================================================
// HELPER: Create a FeatureExtractor for use in tests
// ============================================================================

fn extractor() -> FeatureExtractor {
    FeatureExtractor::new()
}

/// Helper to build a simple HirFunction with a pointer parameter and body.
fn make_func_with_body(param_name: &str, body: Vec<HirStatement>) -> HirFunction {
    HirFunction::new_with_body(
        "test_fn".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            param_name.to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        body,
    )
}

// ============================================================================
// TESTS: count_stmt_accesses - Assignment branch
// ============================================================================

#[test]
fn count_stmt_accesses_assignment_write_to_target_var() {
    // Assignment where target == var_name: should register a write
    let ext = extractor();
    let stmt = HirStatement::Assignment {
        target: "x".to_string(),
        value: HirExpression::IntLiteral(42),
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.write_count, 1);
}

#[test]
fn count_stmt_accesses_assignment_read_from_value() {
    // Assignment where value uses var_name but target != var_name
    let ext = extractor();
    let stmt = HirStatement::Assignment {
        target: "y".to_string(),
        value: HirExpression::Variable("x".to_string()),
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    // value uses x -> read=1, target != x && expr_uses_var(value, x) -> reads += 1 = 2
    assert_eq!(features.read_count, 2);
    assert_eq!(features.write_count, 0);
}

#[test]
fn count_stmt_accesses_assignment_write_and_read_same_var() {
    // Assignment: x = x + 1 (target is x, value uses x)
    let ext = extractor();
    let stmt = HirStatement::Assignment {
        target: "x".to_string(),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        },
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    // value uses x -> reads = 1, target == x -> writes = 1
    // target == x so the second condition (target != x && ...) is false, no extra read
    assert_eq!(features.read_count, 1);
    assert_eq!(features.write_count, 1);
}

#[test]
fn count_stmt_accesses_assignment_no_var_involvement() {
    // Assignment where neither target nor value reference our var
    let ext = extractor();
    let stmt = HirStatement::Assignment {
        target: "y".to_string(),
        value: HirExpression::IntLiteral(10),
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.read_count, 0);
    assert_eq!(features.write_count, 0);
}

#[test]
fn count_stmt_accesses_assignment_value_uses_var_deeply_nested() {
    // Assignment: y = *(x + 1) -- value has nested usage of x
    let ext = extractor();
    let stmt = HirStatement::Assignment {
        target: "y".to_string(),
        value: HirExpression::Dereference(Box::new(HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        })),
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    // value uses x -> reads=1, target != x && value uses x -> reads += 1 = 2
    assert_eq!(features.read_count, 2);
    assert_eq!(features.write_count, 0);
}

// ============================================================================
// TESTS: count_stmt_accesses - DerefAssignment branch
// ============================================================================

#[test]
fn count_stmt_accesses_deref_assignment_target_uses_var() {
    // *x = 42: target expr uses x -> read=1, write=1
    let ext = extractor();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("x".to_string()),
        value: HirExpression::IntLiteral(42),
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.read_count, 1);
    assert_eq!(features.write_count, 1);
}

#[test]
fn count_stmt_accesses_deref_assignment_value_uses_var() {
    // *y = x: value uses x -> read=1, target does not use x -> write=0
    let ext = extractor();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("y".to_string()),
        value: HirExpression::Variable("x".to_string()),
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.read_count, 1);
    assert_eq!(features.write_count, 0);
}

#[test]
fn count_stmt_accesses_deref_assignment_both_use_var() {
    // *x = x: both target and value use x -> read=1, write=1
    let ext = extractor();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("x".to_string()),
        value: HirExpression::Variable("x".to_string()),
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.read_count, 1);
    assert_eq!(features.write_count, 1);
}

#[test]
fn count_stmt_accesses_deref_assignment_neither_uses_var() {
    // *y = z: neither uses x
    let ext = extractor();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::Variable("y".to_string()),
        value: HirExpression::Variable("z".to_string()),
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.read_count, 0);
    assert_eq!(features.write_count, 0);
}

// ============================================================================
// TESTS: count_stmt_accesses - If branch
// ============================================================================

#[test]
fn count_stmt_accesses_if_condition_uses_var() {
    // if (x > 0) { } -- condition uses x -> read=1
    let ext = extractor();
    let stmt = HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        then_block: vec![],
        else_block: None,
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.read_count, 1);
    assert_eq!(features.write_count, 0);
}

#[test]
fn count_stmt_accesses_if_condition_does_not_use_var() {
    // if (y > 0) { } -- condition doesn't use x
    let ext = extractor();
    let stmt = HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("y".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        then_block: vec![],
        else_block: None,
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.read_count, 0);
    assert_eq!(features.write_count, 0);
}

#[test]
fn count_stmt_accesses_if_then_block_uses_var() {
    // if (true) { x = 1; } -- then_block writes x
    let ext = extractor();
    let stmt = HirStatement::If {
        condition: HirExpression::IntLiteral(1),
        then_block: vec![HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::IntLiteral(1),
        }],
        else_block: None,
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.write_count, 1);
}

#[test]
fn count_stmt_accesses_if_else_block_uses_var() {
    // if (true) { } else { x = 2; }
    let ext = extractor();
    let stmt = HirStatement::If {
        condition: HirExpression::IntLiteral(1),
        then_block: vec![],
        else_block: Some(vec![HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::IntLiteral(2),
        }]),
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.write_count, 1);
}

#[test]
fn count_stmt_accesses_if_both_branches_use_var() {
    // if (x) { x = 1; } else { y = x; }
    let ext = extractor();
    let stmt = HirStatement::If {
        condition: HirExpression::Variable("x".to_string()),
        then_block: vec![HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::IntLiteral(1),
        }],
        else_block: Some(vec![HirStatement::Assignment {
            target: "y".to_string(),
            value: HirExpression::Variable("x".to_string()),
        }]),
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    // condition read=1, then_block write=1, else_block read=2 (value uses x + target!=x extra)
    assert!(features.read_count >= 1);
    assert!(features.write_count >= 1);
}

#[test]
fn count_stmt_accesses_if_no_else_block() {
    // if (y) { } -- no else, var not used anywhere
    let ext = extractor();
    let stmt = HirStatement::If {
        condition: HirExpression::Variable("y".to_string()),
        then_block: vec![],
        else_block: None,
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.read_count, 0);
    assert_eq!(features.write_count, 0);
}

#[test]
fn count_stmt_accesses_if_nested_statements_in_then() {
    // if (true) { y = x; z = x; } -- two reads of x in then_block
    let ext = extractor();
    let stmt = HirStatement::If {
        condition: HirExpression::IntLiteral(1),
        then_block: vec![
            HirStatement::Assignment {
                target: "y".to_string(),
                value: HirExpression::Variable("x".to_string()),
            },
            HirStatement::Assignment {
                target: "z".to_string(),
                value: HirExpression::Variable("x".to_string()),
            },
        ],
        else_block: None,
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    // Each assignment: value uses x -> read=1, target != x && value uses x -> read += 1 = 2 per stmt
    assert_eq!(features.read_count, 4);
}

// ============================================================================
// TESTS: count_stmt_accesses - Wildcard/other branch
// ============================================================================

#[test]
fn count_stmt_accesses_return_statement_falls_through() {
    // Return falls into wildcard -> (0, 0)
    let ext = extractor();
    let stmt = HirStatement::Return(Some(HirExpression::Variable("x".to_string())));
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    // count_stmt_accesses returns (0,0) for Return, but check_escape may use it
    // The read_count and write_count from count_accesses should be 0
    assert_eq!(features.read_count, 0);
    assert_eq!(features.write_count, 0);
}

#[test]
fn count_stmt_accesses_while_statement_falls_through() {
    // While falls into wildcard -> (0, 0)
    let ext = extractor();
    let stmt = HirStatement::While {
        condition: HirExpression::Variable("x".to_string()),
        body: vec![HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::IntLiteral(0),
        }],
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    // While is in the wildcard arm for count_stmt_accesses -> (0,0)
    assert_eq!(features.read_count, 0);
    assert_eq!(features.write_count, 0);
}

#[test]
fn count_stmt_accesses_break_statement_falls_through() {
    let ext = extractor();
    let stmt = HirStatement::Break;
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.read_count, 0);
    assert_eq!(features.write_count, 0);
}

#[test]
fn count_stmt_accesses_continue_statement_falls_through() {
    let ext = extractor();
    let stmt = HirStatement::Continue;
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.read_count, 0);
    assert_eq!(features.write_count, 0);
}

#[test]
fn count_stmt_accesses_variable_declaration_falls_through() {
    let ext = extractor();
    let stmt = HirStatement::VariableDeclaration {
        name: "y".to_string(),
        var_type: HirType::Int,
        initializer: Some(HirExpression::Variable("x".to_string())),
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.read_count, 0);
    assert_eq!(features.write_count, 0);
}

#[test]
fn count_stmt_accesses_expression_stmt_falls_through() {
    let ext = extractor();
    let stmt = HirStatement::Expression(HirExpression::Variable("x".to_string()));
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.read_count, 0);
    assert_eq!(features.write_count, 0);
}

#[test]
fn count_stmt_accesses_free_stmt_falls_through() {
    let ext = extractor();
    let stmt = HirStatement::Free {
        pointer: HirExpression::Variable("x".to_string()),
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    // Free is in wildcard for count_stmt_accesses -> (0,0), but counted via count_deallocations
    assert_eq!(features.read_count, 0);
    assert_eq!(features.write_count, 0);
}

// ============================================================================
// TESTS: count_accesses (wrapper that iterates body)
// ============================================================================

#[test]
fn count_accesses_empty_body() {
    let ext = extractor();
    let func = make_func_with_body("x", vec![]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.read_count, 0);
    assert_eq!(features.write_count, 0);
}

#[test]
fn count_accesses_multiple_statements_accumulate() {
    let ext = extractor();
    let body = vec![
        HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::IntLiteral(1),
        },
        HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::IntLiteral(2),
        },
        HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::IntLiteral(3),
        },
    ];
    let func = make_func_with_body("x", body);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.write_count, 3);
}

#[test]
fn count_accesses_mixed_reads_and_writes() {
    let ext = extractor();
    let body = vec![
        // Write to x
        HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::IntLiteral(0),
        },
        // Read from x (y = x): read=2 due to the double-counting
        HirStatement::Assignment {
            target: "y".to_string(),
            value: HirExpression::Variable("x".to_string()),
        },
    ];
    let func = make_func_with_body("x", body);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.write_count, 1);
    assert_eq!(features.read_count, 2);
}

// ============================================================================
// TESTS: expr_uses_var (exercises the recursive expression traversal)
// ============================================================================

#[test]
fn expr_uses_var_variable_match() {
    let ext = extractor();
    let stmt = HirStatement::Assignment {
        target: "y".to_string(),
        value: HirExpression::Variable("x".to_string()),
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert!(features.read_count > 0);
}

#[test]
fn expr_uses_var_variable_no_match() {
    let ext = extractor();
    let stmt = HirStatement::Assignment {
        target: "y".to_string(),
        value: HirExpression::Variable("z".to_string()),
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.read_count, 0);
}

#[test]
fn expr_uses_var_dereference() {
    // y = *x
    let ext = extractor();
    let stmt = HirStatement::Assignment {
        target: "y".to_string(),
        value: HirExpression::Dereference(Box::new(HirExpression::Variable("x".to_string()))),
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert!(features.read_count > 0);
}

#[test]
fn expr_uses_var_address_of() {
    // y = &x
    let ext = extractor();
    let stmt = HirStatement::Assignment {
        target: "y".to_string(),
        value: HirExpression::AddressOf(Box::new(HirExpression::Variable("x".to_string()))),
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert!(features.read_count > 0);
}

#[test]
fn expr_uses_var_binary_op_left() {
    // y = x + 1
    let ext = extractor();
    let stmt = HirStatement::Assignment {
        target: "y".to_string(),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(1)),
        },
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert!(features.read_count > 0);
}

#[test]
fn expr_uses_var_binary_op_right() {
    // y = 1 + x
    let ext = extractor();
    let stmt = HirStatement::Assignment {
        target: "y".to_string(),
        value: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::IntLiteral(1)),
            right: Box::new(HirExpression::Variable("x".to_string())),
        },
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert!(features.read_count > 0);
}

#[test]
fn expr_uses_var_unary_op() {
    // y = -x
    let ext = extractor();
    let stmt = HirStatement::Assignment {
        target: "y".to_string(),
        value: HirExpression::UnaryOp {
            op: UnaryOperator::Minus,
            operand: Box::new(HirExpression::Variable("x".to_string())),
        },
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert!(features.read_count > 0);
}

#[test]
fn expr_uses_var_array_index_array() {
    // y = x[0]
    let ext = extractor();
    let stmt = HirStatement::Assignment {
        target: "y".to_string(),
        value: HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("x".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
        },
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert!(features.read_count > 0);
}

#[test]
fn expr_uses_var_array_index_index() {
    // y = arr[x]
    let ext = extractor();
    let stmt = HirStatement::Assignment {
        target: "y".to_string(),
        value: HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::Variable("x".to_string())),
        },
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert!(features.read_count > 0);
}

#[test]
fn expr_uses_var_function_call_with_arg() {
    // y = foo(x)
    let ext = extractor();
    let stmt = HirStatement::Assignment {
        target: "y".to_string(),
        value: HirExpression::FunctionCall {
            function: "foo".to_string(),
            arguments: vec![HirExpression::Variable("x".to_string())],
        },
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert!(features.read_count > 0);
}

#[test]
fn expr_uses_var_function_call_no_matching_arg() {
    // y = foo(z)
    let ext = extractor();
    let stmt = HirStatement::Assignment {
        target: "y".to_string(),
        value: HirExpression::FunctionCall {
            function: "foo".to_string(),
            arguments: vec![HirExpression::Variable("z".to_string())],
        },
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.read_count, 0);
}

#[test]
fn expr_uses_var_is_not_null() {
    // if (x != NULL) -> IsNotNull(Variable("x"))
    let ext = extractor();
    let stmt = HirStatement::If {
        condition: HirExpression::IsNotNull(Box::new(HirExpression::Variable("x".to_string()))),
        then_block: vec![],
        else_block: None,
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    // condition uses x via IsNotNull -> read=1
    assert_eq!(features.read_count, 1);
}

#[test]
fn expr_uses_var_int_literal_no_match() {
    // y = 42: no var usage
    let ext = extractor();
    let stmt = HirStatement::Assignment {
        target: "y".to_string(),
        value: HirExpression::IntLiteral(42),
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.read_count, 0);
}

// ============================================================================
// TESTS: count_null_checks_in_stmt
// ============================================================================

#[test]
fn null_check_is_not_null_expression() {
    // if (x != NULL) { } -- is_null_check via IsNotNull
    let ext = extractor();
    let body = vec![HirStatement::If {
        condition: HirExpression::IsNotNull(Box::new(HirExpression::Variable("x".to_string()))),
        then_block: vec![],
        else_block: None,
    }];
    let func = make_func_with_body("x", body);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.null_checks, 1);
}

#[test]
fn null_check_binary_op_var_eq_null() {
    // if (x == NULL) { } -- BinaryOp with NullLiteral on right
    let ext = extractor();
    let body = vec![HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::NullLiteral),
        },
        then_block: vec![],
        else_block: None,
    }];
    let func = make_func_with_body("x", body);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.null_checks, 1);
}

#[test]
fn null_check_binary_op_null_eq_var() {
    // if (NULL == x) { } -- BinaryOp with NullLiteral on left
    let ext = extractor();
    let body = vec![HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::NullLiteral),
            right: Box::new(HirExpression::Variable("x".to_string())),
        },
        then_block: vec![],
        else_block: None,
    }];
    let func = make_func_with_body("x", body);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.null_checks, 1);
}

#[test]
fn null_check_not_a_null_check() {
    // if (x > 0) { } -- not a null check
    let ext = extractor();
    let body = vec![HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::GreaterThan,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        then_block: vec![],
        else_block: None,
    }];
    let func = make_func_with_body("x", body);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.null_checks, 0);
}

#[test]
fn null_check_nested_in_then_block() {
    // if (true) { if (x != NULL) { } }
    let ext = extractor();
    let body = vec![HirStatement::If {
        condition: HirExpression::IntLiteral(1),
        then_block: vec![HirStatement::If {
            condition: HirExpression::IsNotNull(Box::new(HirExpression::Variable(
                "x".to_string(),
            ))),
            then_block: vec![],
            else_block: None,
        }],
        else_block: None,
    }];
    let func = make_func_with_body("x", body);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.null_checks, 1);
}

#[test]
fn null_check_nested_in_else_block() {
    // if (true) { } else { if (x == NULL) { } }
    let ext = extractor();
    let body = vec![HirStatement::If {
        condition: HirExpression::IntLiteral(1),
        then_block: vec![],
        else_block: Some(vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::NotEqual,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::NullLiteral),
            },
            then_block: vec![],
            else_block: None,
        }]),
    }];
    let func = make_func_with_body("x", body);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.null_checks, 1);
}

#[test]
fn null_check_in_while_condition() {
    // while (x != NULL) { }
    let ext = extractor();
    let body = vec![HirStatement::While {
        condition: HirExpression::IsNotNull(Box::new(HirExpression::Variable("x".to_string()))),
        body: vec![],
    }];
    let func = make_func_with_body("x", body);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.null_checks, 1);
}

#[test]
fn null_check_in_while_body() {
    // while (true) { if (x == NULL) break; }
    let ext = extractor();
    let body = vec![HirStatement::While {
        condition: HirExpression::IntLiteral(1),
        body: vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::Equal,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::NullLiteral),
            },
            then_block: vec![HirStatement::Break],
            else_block: None,
        }],
    }];
    let func = make_func_with_body("x", body);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.null_checks, 1);
}

#[test]
fn null_check_multiple_checks() {
    // Multiple null checks in sequence
    let ext = extractor();
    let body = vec![
        HirStatement::If {
            condition: HirExpression::IsNotNull(Box::new(HirExpression::Variable(
                "x".to_string(),
            ))),
            then_block: vec![],
            else_block: None,
        },
        HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::Equal,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::NullLiteral),
            },
            then_block: vec![],
            else_block: None,
        },
    ];
    let func = make_func_with_body("x", body);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.null_checks, 2);
}

#[test]
fn null_check_wildcard_stmt_returns_zero() {
    // Assignment is wildcard for count_null_checks_in_stmt -> 0
    let ext = extractor();
    let body = vec![HirStatement::Assignment {
        target: "x".to_string(),
        value: HirExpression::NullLiteral,
    }];
    let func = make_func_with_body("x", body);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.null_checks, 0);
}

// ============================================================================
// TESTS: Complex/combined scenarios
// ============================================================================

#[test]
fn combined_reads_writes_and_null_checks() {
    let ext = extractor();
    let body = vec![
        // Write to x
        HirStatement::Assignment {
            target: "x".to_string(),
            value: HirExpression::IntLiteral(0),
        },
        // Null check on x
        HirStatement::If {
            condition: HirExpression::IsNotNull(Box::new(HirExpression::Variable(
                "x".to_string(),
            ))),
            then_block: vec![
                // Deref write through x
                HirStatement::DerefAssignment {
                    target: HirExpression::Variable("x".to_string()),
                    value: HirExpression::IntLiteral(42),
                },
            ],
            else_block: None,
        },
    ];
    let func = make_func_with_body("x", body);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.write_count, 2); // 1 assignment + 1 deref assignment
    assert_eq!(features.null_checks, 1);
    // read: if condition uses x -> 1, deref target uses x -> 1
    assert_eq!(features.read_count, 2);
}

#[test]
fn deeply_nested_if_else_chain() {
    let ext = extractor();
    let stmt = HirStatement::If {
        condition: HirExpression::Variable("x".to_string()),
        then_block: vec![HirStatement::If {
            condition: HirExpression::Variable("x".to_string()),
            then_block: vec![HirStatement::Assignment {
                target: "x".to_string(),
                value: HirExpression::IntLiteral(0),
            }],
            else_block: Some(vec![HirStatement::Assignment {
                target: "y".to_string(),
                value: HirExpression::Variable("x".to_string()),
            }]),
        }],
        else_block: None,
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    // outer condition: read=1
    // inner condition: read=1
    // inner then: write=1
    // inner else: y = x -> read=2 (value + double count)
    assert!(features.read_count >= 4);
    assert_eq!(features.write_count, 1);
}

#[test]
fn deref_assignment_with_nested_expressions() {
    // *(x + offset) = value
    let ext = extractor();
    let stmt = HirStatement::DerefAssignment {
        target: HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(4)),
        },
        value: HirExpression::IntLiteral(99),
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    // target uses x -> read=1, write=1
    assert_eq!(features.read_count, 1);
    assert_eq!(features.write_count, 1);
}

#[test]
fn multiple_deref_assignments_accumulate() {
    let ext = extractor();
    let body = vec![
        HirStatement::DerefAssignment {
            target: HirExpression::Variable("x".to_string()),
            value: HirExpression::IntLiteral(1),
        },
        HirStatement::DerefAssignment {
            target: HirExpression::Variable("x".to_string()),
            value: HirExpression::IntLiteral(2),
        },
    ];
    let func = make_func_with_body("x", body);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.read_count, 2);
    assert_eq!(features.write_count, 2);
}

#[test]
fn if_with_deref_assignment_in_else() {
    let ext = extractor();
    let stmt = HirStatement::If {
        condition: HirExpression::IntLiteral(0),
        then_block: vec![],
        else_block: Some(vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("x".to_string()),
            value: HirExpression::Variable("x".to_string()),
        }]),
    };
    let func = make_func_with_body("x", vec![stmt]);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    // else block: deref target uses x || value uses x -> read=1, target uses x -> write=1
    assert_eq!(features.read_count, 1);
    assert_eq!(features.write_count, 1);
}

#[test]
fn null_check_is_null_check_wildcard() {
    // is_null_check with a non-matching expression (IntLiteral) falls through wildcard -> false
    let ext = extractor();
    let body = vec![HirStatement::If {
        condition: HirExpression::IntLiteral(0),
        then_block: vec![],
        else_block: None,
    }];
    let func = make_func_with_body("x", body);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.null_checks, 0);
}

#[test]
fn null_check_binary_op_neither_is_null() {
    // BinaryOp but neither side is NullLiteral -- not a null check
    let ext = extractor();
    let body = vec![HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(0)),
        },
        then_block: vec![],
        else_block: None,
    }];
    let func = make_func_with_body("x", body);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.null_checks, 0);
}

#[test]
fn null_check_binary_op_null_but_wrong_var() {
    // if (y == NULL) -- null check but for different var
    let ext = extractor();
    let body = vec![HirStatement::If {
        condition: HirExpression::BinaryOp {
            op: BinaryOperator::Equal,
            left: Box::new(HirExpression::Variable("y".to_string())),
            right: Box::new(HirExpression::NullLiteral),
        },
        then_block: vec![],
        else_block: None,
    }];
    let func = make_func_with_body("x", body);
    let features = ext.extract_for_parameter(&func, "x").unwrap();
    assert_eq!(features.null_checks, 0);
}
