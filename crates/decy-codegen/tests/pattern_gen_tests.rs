//! RED phase tests for pattern matching generation from tag checks (DECY-082).

use decy_codegen::pattern_gen::PatternGenerator;
use decy_hir::{BinaryOperator, HirExpression, HirStatement};

#[test]
fn test_simple_tag_check_to_match() {
    // C: if (v.tag == INT) return v.data.i;
    // Rust: match v { Value::Int(i) => return i, _ => {} }

    let tag_check = HirStatement::If {
        condition: HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT".to_string())), // Enum constant
        },
        then_block: vec![HirStatement::Return(Some(HirExpression::FieldAccess {
            object: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "data".to_string(),
            }),
            field: "i".to_string(),
        }))],
        else_block: None,
    };

    let generator = PatternGenerator::new();
    let match_expr = generator.transform_tag_check(&tag_check);

    assert!(match_expr.contains("match v"));
    assert!(match_expr.contains("Value::Int(i)"));
    assert!(match_expr.contains("=> return i"));
}

#[test]
fn test_tag_check_with_multiple_variants() {
    // C:
    // if (v.tag == INT) return v.data.i;
    // else if (v.tag == FLOAT) return v.data.f;

    let tag_check = HirStatement::If {
        condition: HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT".to_string())),
        },
        then_block: vec![HirStatement::Return(Some(HirExpression::FieldAccess {
            object: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "data".to_string(),
            }),
            field: "i".to_string(),
        }))],
        else_block: Some(vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                left: Box::new(HirExpression::FieldAccess {
                    object: Box::new(HirExpression::Variable("v".to_string())),
                    field: "tag".to_string(),
                }),
                op: BinaryOperator::Equal,
                right: Box::new(HirExpression::Variable("FLOAT".to_string())),
            },
            then_block: vec![HirStatement::Return(Some(HirExpression::FieldAccess {
                object: Box::new(HirExpression::FieldAccess {
                    object: Box::new(HirExpression::Variable("v".to_string())),
                    field: "data".to_string(),
                }),
                field: "f".to_string(),
            }))],
            else_block: None,
        }]),
    };

    let generator = PatternGenerator::new();
    let match_expr = generator.transform_tag_check(&tag_check);

    assert!(match_expr.contains("Value::Int(i)"));
    assert!(match_expr.contains("Value::Float(f)"));
}

#[test]
fn test_exhaustive_match_with_default_case() {
    // Should generate a wildcard pattern for non-exhaustive tag checks
    let tag_check = HirStatement::If {
        condition: HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT".to_string())),
        },
        then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
        else_block: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
            1,
        )))]),
    };

    let generator = PatternGenerator::new();
    let match_expr = generator.transform_tag_check(&tag_check);

    // Should have a catch-all pattern
    assert!(match_expr.contains("_") || match_expr.contains("else"));
}

#[test]
fn test_variant_name_capitalization() {
    // Tag values should be converted to PascalCase variant names
    let tag_check = HirStatement::If {
        condition: HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT_VALUE".to_string())),
        },
        then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(42)))],
        else_block: None,
    };

    let generator = PatternGenerator::new();
    let match_expr = generator.transform_tag_check(&tag_check);

    // Should convert INT_VALUE to IntValue
    assert!(
        match_expr.contains("IntValue") || match_expr.contains("Int"),
        "Expected variant name capitalization"
    );
}

#[test]
fn test_union_field_to_binding() {
    // Union field access should become pattern binding
    // C: v.data.int_value → Rust: Value::Int(int_value)

    let tag_check = HirStatement::If {
        condition: HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT".to_string())),
        },
        then_block: vec![HirStatement::Expression(HirExpression::FieldAccess {
            object: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "data".to_string(),
            }),
            field: "int_value".to_string(),
        })],
        else_block: None,
    };

    let generator = PatternGenerator::new();
    let match_expr = generator.transform_tag_check(&tag_check);

    assert!(match_expr.contains("int_value"));
}

#[test]
fn test_nested_tag_checks() {
    // Nested if-else-if chains should all become match arms
    let tag_check = HirStatement::If {
        condition: HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("node".to_string())),
                field: "type".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("LITERAL".to_string())),
        },
        then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
        else_block: Some(vec![HirStatement::If {
            condition: HirExpression::BinaryOp {
                left: Box::new(HirExpression::FieldAccess {
                    object: Box::new(HirExpression::Variable("node".to_string())),
                    field: "type".to_string(),
                }),
                op: BinaryOperator::Equal,
                right: Box::new(HirExpression::Variable("BINARY_OP".to_string())),
            },
            then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(2)))],
            else_block: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
                3,
            )))]),
        }]),
    };

    let generator = PatternGenerator::new();
    let match_expr = generator.transform_tag_check(&tag_check);

    assert!(match_expr.contains("match node"));
    assert!(match_expr.contains("Literal") || match_expr.contains("LITERAL"));
    assert!(match_expr.contains("BinaryOp") || match_expr.contains("BINARY_OP"));
}

#[test]
fn test_preserves_non_tag_checks() {
    // Non-tag comparisons should not be transformed
    let normal_if = HirStatement::If {
        condition: HirExpression::BinaryOp {
            left: Box::new(HirExpression::Variable("x".to_string())),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::IntLiteral(5)),
        },
        then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
        else_block: None,
    };

    let generator = PatternGenerator::new();
    let result = generator.transform_tag_check(&normal_if);

    // Should return empty or indicate no transformation
    assert!(
        result.is_empty() || result.contains("if"),
        "Non-tag checks should not be transformed to match"
    );
}

#[test]
fn test_match_expression_syntax_valid() {
    let tag_check = HirStatement::If {
        condition: HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT".to_string())),
        },
        then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(42)))],
        else_block: None,
    };

    let generator = PatternGenerator::new();
    let match_expr = generator.transform_tag_check(&tag_check);

    // Basic syntax checks
    assert!(match_expr.contains("match"));
    assert!(match_expr.contains("{"));
    assert!(match_expr.contains("}"));
    assert!(match_expr.contains("=>"));
}
