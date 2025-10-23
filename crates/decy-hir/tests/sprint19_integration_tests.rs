/// Sprint 19 HIR Integration Tests
/// RED phase for DECY-064: HIR/Codegen Integration
///
/// These tests validate that Sprint 19 parser features can be represented in HIR:
/// - Cast expressions
/// - Compound literals
/// - Global variables (with storage class specifiers)

use decy_hir::{HirExpression, HirType};

#[test]
fn test_hir_cast_expression_creation() {
    // Test creating a Cast expression in HIR
    // C: (int)3.14
    // HIR: Cast { target_type: Int, expr: FloatLiteral(3.14) }

    let cast_expr = HirExpression::Cast {
        target_type: HirType::Int,
        expr: Box::new(HirExpression::IntLiteral(3)), // Simplified
    };

    match cast_expr {
        HirExpression::Cast { target_type, expr } => {
            assert_eq!(target_type, HirType::Int);
            assert!(matches!(*expr, HirExpression::IntLiteral(3)));
        }
        _ => panic!("Expected Cast expression"),
    }
}

#[test]
fn test_hir_cast_pointer_types() {
    // Test casting pointer types
    // C: (void*)ptr
    // HIR: Cast { target_type: Pointer(Void), expr: Variable("ptr") }

    let cast_expr = HirExpression::Cast {
        target_type: HirType::Pointer(Box::new(HirType::Void)),
        expr: Box::new(HirExpression::Variable("ptr".to_string())),
    };

    match cast_expr {
        HirExpression::Cast { target_type, .. } => {
            assert!(matches!(target_type, HirType::Pointer(_)));
        }
        _ => panic!("Expected Cast expression"),
    }
}

#[test]
fn test_hir_compound_literal_struct() {
    // Test creating a compound literal in HIR
    // C: (struct Point){10, 20}
    // HIR: CompoundLiteral { literal_type: Named("Point"), initializers: [...] }

    let compound_expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Struct("Point".to_string()),
        initializers: vec![
            HirExpression::IntLiteral(10),
            HirExpression::IntLiteral(20),
        ],
    };

    match compound_expr {
        HirExpression::CompoundLiteral { literal_type, initializers } => {
            assert_eq!(literal_type, HirType::Struct("Point".to_string()));
            assert_eq!(initializers.len(), 2);
        }
        _ => panic!("Expected CompoundLiteral expression"),
    }
}

#[test]
fn test_hir_compound_literal_array() {
    // Test array compound literal
    // C: (int[]){1, 2, 3, 4, 5}
    // HIR: CompoundLiteral { literal_type: Array(Int, 5), initializers: [...] }

    let compound_expr = HirExpression::CompoundLiteral {
        literal_type: HirType::Array {
            element_type: Box::new(HirType::Int),
            size: Some(5),
        },
        initializers: vec![
            HirExpression::IntLiteral(1),
            HirExpression::IntLiteral(2),
            HirExpression::IntLiteral(3),
            HirExpression::IntLiteral(4),
            HirExpression::IntLiteral(5),
        ],
    };

    match compound_expr {
        HirExpression::CompoundLiteral { literal_type, initializers } => {
            assert!(matches!(literal_type, HirType::Array { .. }));
            assert_eq!(initializers.len(), 5);
        }
        _ => panic!("Expected CompoundLiteral expression"),
    }
}

#[test]
fn test_hir_nested_cast_and_compound_literal() {
    // Test combining cast and compound literal
    // C: (struct Point*)(malloc(sizeof(struct Point)))
    // Could also use compound literal: &(struct Point){0, 0}

    let nested_expr = HirExpression::Cast {
        target_type: HirType::Pointer(Box::new(HirType::Struct("Point".to_string()))),
        expr: Box::new(HirExpression::CompoundLiteral {
            literal_type: HirType::Struct("Point".to_string()),
            initializers: vec![
                HirExpression::IntLiteral(0),
                HirExpression::IntLiteral(0),
            ],
        }),
    };

    // Just verify it compiles with the right structure
    match nested_expr {
        HirExpression::Cast { expr, .. } => {
            assert!(matches!(*expr, HirExpression::CompoundLiteral { .. }));
        }
        _ => panic!("Expected Cast with CompoundLiteral inside"),
    }
}

#[test]
fn test_hir_cast_in_binary_operation() {
    // Test cast as part of a larger expression
    // C: (int)x + 10
    // HIR: BinaryOp { Add, Cast(...), IntLiteral(10) }

    use decy_hir::BinaryOperator;

    let expr = HirExpression::BinaryOp {
        op: BinaryOperator::Add,
        left: Box::new(HirExpression::Cast {
            target_type: HirType::Int,
            expr: Box::new(HirExpression::Variable("x".to_string())),
        }),
        right: Box::new(HirExpression::IntLiteral(10)),
    };

    match expr {
        HirExpression::BinaryOp { left, .. } => {
            assert!(matches!(*left, HirExpression::Cast { .. }));
        }
        _ => panic!("Expected BinaryOp"),
    }
}
