//! DECY-196: HIR optimization passes.
//!
//! Three optimization passes run between ownership inference and codegen:
//! 1. **Constant folding**: Evaluate literal arithmetic at compile time
//! 2. **Dead branch removal**: Eliminate `if(1)` / `if(0)` branches
//! 3. **Temporary elimination**: Remove single-use let bindings
//!
//! Passes run in a fixed-point loop (max 3 iterations).
//!
//! # Examples
//!
//! ```
//! use decy_hir::{HirFunction, HirType, HirStatement, HirExpression, BinaryOperator};
//!
//! // Create function with `return 2 + 3;`
//! let func = HirFunction::new_with_body(
//!     "test".to_string(),
//!     HirType::Int,
//!     vec![],
//!     vec![HirStatement::Return(Some(HirExpression::BinaryOp {
//!         op: BinaryOperator::Add,
//!         left: Box::new(HirExpression::IntLiteral(2)),
//!         right: Box::new(HirExpression::IntLiteral(3)),
//!     }))],
//! );
//!
//! let optimized = decy_core::optimize::optimize_function(&func);
//! // After constant folding: `return 5;`
//! assert_eq!(optimized.body().len(), 1);
//! ```

use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirStatement};
#[cfg(test)]
use decy_hir::HirType;

/// Maximum number of fixed-point iterations.
const MAX_ITERATIONS: usize = 3;

/// Run all optimization passes on a function.
///
/// Runs constant folding, dead branch removal, and temporary elimination
/// in a fixed-point loop until no more changes are made or MAX_ITERATIONS is reached.
pub fn optimize_function(func: &HirFunction) -> HirFunction {
    let mut body = func.body().to_vec();
    let mut changed = true;
    let mut iterations = 0;

    while changed && iterations < MAX_ITERATIONS {
        changed = false;
        let prev = body.clone();

        body = body.into_iter().map(fold_constants_stmt).collect();
        body = remove_dead_branches(body);
        body = eliminate_temporaries(body);

        if body != prev {
            changed = true;
        }
        iterations += 1;
    }

    HirFunction::new_with_body(
        func.name().to_string(),
        func.return_type().clone(),
        func.parameters().to_vec(),
        body,
    )
}

// ============================================================================
// Pass 1: Constant Folding
// ============================================================================

/// Fold constant expressions in a statement.
fn fold_constants_stmt(stmt: HirStatement) -> HirStatement {
    match stmt {
        HirStatement::VariableDeclaration {
            name,
            var_type,
            initializer,
        } => HirStatement::VariableDeclaration {
            name,
            var_type,
            initializer: initializer.map(fold_constants_expr),
        },
        HirStatement::Return(expr) => HirStatement::Return(expr.map(fold_constants_expr)),
        HirStatement::Assignment { target, value } => HirStatement::Assignment {
            target,
            value: fold_constants_expr(value),
        },
        HirStatement::If {
            condition,
            then_block,
            else_block,
        } => HirStatement::If {
            condition: fold_constants_expr(condition),
            then_block: then_block.into_iter().map(fold_constants_stmt).collect(),
            else_block: else_block
                .map(|block| block.into_iter().map(fold_constants_stmt).collect()),
        },
        HirStatement::While { condition, body } => HirStatement::While {
            condition: fold_constants_expr(condition),
            body: body.into_iter().map(fold_constants_stmt).collect(),
        },
        HirStatement::For {
            init,
            condition,
            increment,
            body,
        } => HirStatement::For {
            init: init.into_iter().map(fold_constants_stmt).collect(),
            condition: fold_constants_expr(condition),
            increment: increment.into_iter().map(fold_constants_stmt).collect(),
            body: body.into_iter().map(fold_constants_stmt).collect(),
        },
        HirStatement::Expression(expr) => HirStatement::Expression(fold_constants_expr(expr)),
        // Statements without foldable expressions pass through
        other => other,
    }
}

/// Fold constant expressions recursively.
fn fold_constants_expr(expr: HirExpression) -> HirExpression {
    match expr {
        HirExpression::BinaryOp { op, left, right } => {
            let left = fold_constants_expr(*left);
            let right = fold_constants_expr(*right);

            // Try to fold integer arithmetic
            if let (HirExpression::IntLiteral(l), HirExpression::IntLiteral(r)) = (&left, &right) {
                if let Some(result) = fold_int_binary(*l, op, *r) {
                    return HirExpression::IntLiteral(result);
                }
            }

            HirExpression::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        HirExpression::UnaryOp { op, operand } => {
            let operand = fold_constants_expr(*operand);
            if let (decy_hir::UnaryOperator::Minus, HirExpression::IntLiteral(v)) =
                (op, &operand)
            {
                return HirExpression::IntLiteral(-v);
            }
            HirExpression::UnaryOp {
                op,
                operand: Box::new(operand),
            }
        }
        // Recurse into nested expressions
        HirExpression::FunctionCall {
            function,
            arguments,
        } => HirExpression::FunctionCall {
            function,
            arguments: arguments.into_iter().map(fold_constants_expr).collect(),
        },
        other => other,
    }
}

/// Try to evaluate a binary operation on integer literals.
fn fold_int_binary(left: i32, op: BinaryOperator, right: i32) -> Option<i32> {
    match op {
        BinaryOperator::Add => left.checked_add(right),
        BinaryOperator::Subtract => left.checked_sub(right),
        BinaryOperator::Multiply => left.checked_mul(right),
        BinaryOperator::Divide => {
            if right != 0 {
                left.checked_div(right)
            } else {
                None
            }
        }
        BinaryOperator::Modulo => {
            if right != 0 {
                left.checked_rem(right)
            } else {
                None
            }
        }
        BinaryOperator::LeftShift => {
            if (0..32).contains(&right) {
                Some(left << right)
            } else {
                None
            }
        }
        BinaryOperator::RightShift => {
            if (0..32).contains(&right) {
                Some(left >> right)
            } else {
                None
            }
        }
        BinaryOperator::BitwiseAnd => Some(left & right),
        BinaryOperator::BitwiseOr => Some(left | right),
        BinaryOperator::BitwiseXor => Some(left ^ right),
        _ => None,
    }
}

// ============================================================================
// Pass 2: Dead Branch Removal
// ============================================================================

/// Remove dead branches: `if(1) { ... }` → `...`, `if(0) { ... }` → removed.
fn remove_dead_branches(stmts: Vec<HirStatement>) -> Vec<HirStatement> {
    let mut result = Vec::new();

    for stmt in stmts {
        match stmt {
            HirStatement::If {
                condition,
                then_block,
                else_block,
            } => {
                if let Some(always_true) = is_constant_truthy(&condition) {
                    if always_true {
                        // if(1) { body } → body
                        result.extend(
                            then_block
                                .into_iter()
                                .map(fold_constants_stmt)
                                .collect::<Vec<_>>(),
                        );
                    } else if let Some(else_body) = else_block {
                        // if(0) { ... } else { body } → body
                        result.extend(
                            else_body
                                .into_iter()
                                .map(fold_constants_stmt)
                                .collect::<Vec<_>>(),
                        );
                    }
                    // if(0) { ... } with no else → removed entirely
                } else {
                    // Non-constant condition: recurse into both branches
                    result.push(HirStatement::If {
                        condition,
                        then_block: remove_dead_branches(then_block),
                        else_block: else_block.map(remove_dead_branches),
                    });
                }
            }
            HirStatement::While { condition, body } => {
                // while(0) → removed entirely
                if let Some(false) = is_constant_truthy(&condition) {
                    // Dead loop, skip
                } else {
                    result.push(HirStatement::While {
                        condition,
                        body: remove_dead_branches(body),
                    });
                }
            }
            other => result.push(other),
        }
    }

    result
}

/// Check if an expression is a constant truthy/falsy value.
fn is_constant_truthy(expr: &HirExpression) -> Option<bool> {
    match expr {
        HirExpression::IntLiteral(0) => Some(false),
        HirExpression::IntLiteral(_) => Some(true),
        _ => None,
    }
}

// ============================================================================
// Pass 3: Temporary Elimination
// ============================================================================

/// Eliminate single-use temporary variables.
///
/// Pattern: `let tmp = expr; return tmp;` → `return expr;`
///
/// Conservative: does NOT inline malloc/calloc/realloc results since
/// downstream passes (box_transform) need the variable assignment pattern.
fn eliminate_temporaries(stmts: Vec<HirStatement>) -> Vec<HirStatement> {
    if stmts.len() < 2 {
        return stmts;
    }

    let mut result = Vec::new();
    let mut skip_next = false;

    for i in 0..stmts.len() {
        if skip_next {
            skip_next = false;
            continue;
        }

        // Check for pattern: let tmp = expr; return tmp;
        if i + 1 < stmts.len() {
            if let (
                HirStatement::VariableDeclaration {
                    name,
                    initializer: Some(init_expr),
                    ..
                },
                HirStatement::Return(Some(HirExpression::Variable(ret_var))),
            ) = (&stmts[i], &stmts[i + 1])
            {
                if name == ret_var
                    && count_uses(name, &stmts[i + 2..]) == 0
                    && !is_allocation_expr(init_expr)
                {
                    // Replace with direct return
                    result.push(HirStatement::Return(Some(init_expr.clone())));
                    skip_next = true;
                    continue;
                }
            }
        }

        result.push(stmts[i].clone());
    }

    result
}

/// Check if an expression is an allocation (malloc, calloc, realloc, cast wrapping allocation).
/// These should NOT be inlined because box_transform needs the variable pattern.
fn is_allocation_expr(expr: &HirExpression) -> bool {
    match expr {
        HirExpression::Malloc { .. }
        | HirExpression::Calloc { .. }
        | HirExpression::Realloc { .. } => true,
        HirExpression::Cast { expr: inner, .. } => is_allocation_expr(inner),
        HirExpression::FunctionCall { function, .. } => {
            matches!(function.as_str(), "malloc" | "calloc" | "realloc")
        }
        _ => false,
    }
}

/// Count how many times a variable is used in a slice of statements.
fn count_uses(name: &str, stmts: &[HirStatement]) -> usize {
    let mut count = 0;
    for stmt in stmts {
        count += count_uses_in_stmt(name, stmt);
    }
    count
}

/// Count uses of a variable in a single statement.
fn count_uses_in_stmt(name: &str, stmt: &HirStatement) -> usize {
    match stmt {
        HirStatement::Return(Some(expr)) => count_uses_in_expr(name, expr),
        HirStatement::Assignment { value, .. } => count_uses_in_expr(name, value),
        HirStatement::Expression(expr) => count_uses_in_expr(name, expr),
        HirStatement::If {
            condition,
            then_block,
            else_block,
        } => {
            let mut c = count_uses_in_expr(name, condition);
            for s in then_block {
                c += count_uses_in_stmt(name, s);
            }
            if let Some(block) = else_block {
                for s in block {
                    c += count_uses_in_stmt(name, s);
                }
            }
            c
        }
        _ => 0,
    }
}

/// Count uses of a variable in an expression.
fn count_uses_in_expr(name: &str, expr: &HirExpression) -> usize {
    match expr {
        HirExpression::Variable(v) if v == name => 1,
        HirExpression::BinaryOp { left, right, .. } => {
            count_uses_in_expr(name, left) + count_uses_in_expr(name, right)
        }
        HirExpression::FunctionCall { arguments, .. } => {
            arguments.iter().map(|a| count_uses_in_expr(name, a)).sum()
        }
        HirExpression::UnaryOp { operand, .. } => count_uses_in_expr(name, operand),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_folding_add() {
        let expr = HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::IntLiteral(2)),
            right: Box::new(HirExpression::IntLiteral(3)),
        };
        assert_eq!(fold_constants_expr(expr), HirExpression::IntLiteral(5));
    }

    #[test]
    fn test_constant_folding_multiply() {
        let expr = HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::IntLiteral(4)),
            right: Box::new(HirExpression::IntLiteral(5)),
        };
        assert_eq!(fold_constants_expr(expr), HirExpression::IntLiteral(20));
    }

    #[test]
    fn test_constant_folding_nested() {
        // (2 + 3) * 4 → 20
        let expr = HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::IntLiteral(2)),
                right: Box::new(HirExpression::IntLiteral(3)),
            }),
            right: Box::new(HirExpression::IntLiteral(4)),
        };
        assert_eq!(fold_constants_expr(expr), HirExpression::IntLiteral(20));
    }

    #[test]
    fn test_constant_folding_division_by_zero() {
        let expr = HirExpression::BinaryOp {
            op: BinaryOperator::Divide,
            left: Box::new(HirExpression::IntLiteral(10)),
            right: Box::new(HirExpression::IntLiteral(0)),
        };
        // Should not fold (division by zero)
        match fold_constants_expr(expr) {
            HirExpression::BinaryOp { .. } => {} // Expected: not folded
            other => panic!("Expected BinaryOp, got {:?}", other),
        }
    }

    #[test]
    fn test_constant_folding_non_literal() {
        let expr = HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("x".to_string())),
            right: Box::new(HirExpression::IntLiteral(3)),
        };
        // Should not fold (variable operand)
        match fold_constants_expr(expr) {
            HirExpression::BinaryOp { .. } => {} // Expected: not folded
            other => panic!("Expected BinaryOp, got {:?}", other),
        }
    }

    #[test]
    fn test_dead_branch_removal_true() {
        // if(1) { return 42; } → return 42;
        let stmts = vec![HirStatement::If {
            condition: HirExpression::IntLiteral(1),
            then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(42)))],
            else_block: None,
        }];

        let result = remove_dead_branches(stmts);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            HirStatement::Return(Some(HirExpression::IntLiteral(42)))
        );
    }

    #[test]
    fn test_dead_branch_removal_false_no_else() {
        // if(0) { return 42; } → (removed)
        let stmts = vec![HirStatement::If {
            condition: HirExpression::IntLiteral(0),
            then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(42)))],
            else_block: None,
        }];

        let result = remove_dead_branches(stmts);
        assert!(result.is_empty());
    }

    #[test]
    fn test_dead_branch_removal_false_with_else() {
        // if(0) { ... } else { return 99; } → return 99;
        let stmts = vec![HirStatement::If {
            condition: HirExpression::IntLiteral(0),
            then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(42)))],
            else_block: Some(vec![HirStatement::Return(Some(
                HirExpression::IntLiteral(99),
            ))]),
        }];

        let result = remove_dead_branches(stmts);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            HirStatement::Return(Some(HirExpression::IntLiteral(99)))
        );
    }

    #[test]
    fn test_dead_while_zero() {
        // while(0) { ... } → (removed)
        let stmts = vec![HirStatement::While {
            condition: HirExpression::IntLiteral(0),
            body: vec![HirStatement::Break],
        }];

        let result = remove_dead_branches(stmts);
        assert!(result.is_empty());
    }

    #[test]
    fn test_temp_elimination_return() {
        // let tmp = 42; return tmp; → return 42;
        let stmts = vec![
            HirStatement::VariableDeclaration {
                name: "tmp".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(42)),
            },
            HirStatement::Return(Some(HirExpression::Variable("tmp".to_string()))),
        ];

        let result = eliminate_temporaries(stmts);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            HirStatement::Return(Some(HirExpression::IntLiteral(42)))
        );
    }

    #[test]
    fn test_optimize_function_combined() {
        // function with: let x = 2 + 3; if(1) { return x; }
        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Int,
            vec![],
            vec![
                HirStatement::VariableDeclaration {
                    name: "x".to_string(),
                    var_type: HirType::Int,
                    initializer: Some(HirExpression::BinaryOp {
                        op: BinaryOperator::Add,
                        left: Box::new(HirExpression::IntLiteral(2)),
                        right: Box::new(HirExpression::IntLiteral(3)),
                    }),
                },
                HirStatement::If {
                    condition: HirExpression::IntLiteral(1),
                    then_block: vec![HirStatement::Return(Some(HirExpression::Variable(
                        "x".to_string(),
                    )))],
                    else_block: None,
                },
            ],
        );

        let optimized = optimize_function(&func);
        // After optimization: constant folding turns 2+3→5, dead branch removal inlines the if(1)
        // Result: let x = 5; return x; → return 5; (after temp elimination)
        let body = optimized.body();
        assert!(
            body.len() <= 2,
            "Expected at most 2 statements, got {}",
            body.len()
        );
    }

    #[test]
    fn test_unary_minus_folding() {
        let expr = HirExpression::UnaryOp {
            op: decy_hir::UnaryOperator::Minus,
            operand: Box::new(HirExpression::IntLiteral(42)),
        };
        assert_eq!(fold_constants_expr(expr), HirExpression::IntLiteral(-42));
    }

    #[test]
    fn test_bitwise_folding() {
        let expr = HirExpression::BinaryOp {
            op: BinaryOperator::BitwiseAnd,
            left: Box::new(HirExpression::IntLiteral(0xFF)),
            right: Box::new(HirExpression::IntLiteral(0x0F)),
        };
        assert_eq!(fold_constants_expr(expr), HirExpression::IntLiteral(0x0F));
    }
}
