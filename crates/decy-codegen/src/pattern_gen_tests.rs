//! Tests for pattern matching generation (DECY-082).
//!
//! Comprehensive tests to achieve 95%+ coverage.

#[cfg(test)]
mod tests {
    use crate::pattern_gen::PatternGenerator;
    use decy_hir::{BinaryOperator, HirExpression, HirStatement};

    // ========================================================================
    // PatternGenerator::new() and Default tests
    // ========================================================================

    #[test]
    fn test_pattern_generator_new() {
        let gen = PatternGenerator::new();
        let stmt = HirStatement::Break;
        let result = gen.transform_tag_check(&stmt);
        assert!(result.is_empty());
    }

    #[test]
    fn test_pattern_generator_default() {
        let gen = PatternGenerator;
        let stmt = HirStatement::Continue;
        let result = gen.transform_tag_check(&stmt);
        assert!(result.is_empty());
    }

    // ========================================================================
    // transform_tag_check tests
    // ========================================================================

    #[test]
    fn test_transform_tag_check_non_if_statement() {
        let gen = PatternGenerator::new();

        // Test with break
        let result = gen.transform_tag_check(&HirStatement::Break);
        assert!(result.is_empty());

        // Test with continue
        let result = gen.transform_tag_check(&HirStatement::Continue);
        assert!(result.is_empty());

        // Test with return
        let result = gen.transform_tag_check(&HirStatement::Return(None));
        assert!(result.is_empty());
    }

    #[test]
    fn test_transform_tag_check_if_without_tag_comparison() {
        let gen = PatternGenerator::new();

        // If statement without tag comparison (just variable)
        let stmt = HirStatement::If {
            condition: HirExpression::Variable("x".to_string()),
            then_block: vec![],
            else_block: None,
        };

        let result = gen.transform_tag_check(&stmt);
        assert!(result.is_empty());
    }

    #[test]
    fn test_transform_tag_check_if_with_non_equal_comparison() {
        let gen = PatternGenerator::new();

        // If statement with != comparison (not ==)
        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::NotEqual,
            right: Box::new(HirExpression::Variable("INT".to_string())),
        };

        let stmt = HirStatement::If {
            condition,
            then_block: vec![],
            else_block: None,
        };

        let result = gen.transform_tag_check(&stmt);
        assert!(result.is_empty());
    }

    #[test]
    fn test_transform_tag_check_simple_tag_comparison() {
        let gen = PatternGenerator::new();

        // v.tag == INT pattern
        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT".to_string())),
        };

        let stmt = HirStatement::If {
            condition,
            then_block: vec![],
            else_block: None,
        };

        let result = gen.transform_tag_check(&stmt);
        assert!(result.contains("match v"));
        assert!(result.contains("Value::Int"));
    }

    #[test]
    fn test_transform_tag_check_with_union_field_access() {
        let gen = PatternGenerator::new();

        // v.tag == INT with return v.data.i
        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT".to_string())),
        };

        let union_field = HirExpression::FieldAccess {
            object: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "data".to_string(),
            }),
            field: "i".to_string(),
        };

        let stmt = HirStatement::If {
            condition,
            then_block: vec![HirStatement::Return(Some(union_field))],
            else_block: None,
        };

        let result = gen.transform_tag_check(&stmt);
        assert!(result.contains("Value::Int(i)"));
        assert!(result.contains("return i"));
    }

    #[test]
    fn test_transform_tag_check_with_else_block() {
        let gen = PatternGenerator::new();

        // v.tag == INT with else block
        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT".to_string())),
        };

        let stmt = HirStatement::If {
            condition,
            then_block: vec![],
            else_block: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
                -1,
            )))]),
        };

        let result = gen.transform_tag_check(&stmt);
        assert!(result.contains("_ =>"));
    }

    #[test]
    fn test_transform_tag_check_else_if_chain() {
        let gen = PatternGenerator::new();

        // Inner else-if: v.tag == FLOAT
        let inner_condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("FLOAT".to_string())),
        };

        let inner_if = HirStatement::If {
            condition: inner_condition,
            then_block: vec![],
            else_block: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
                -1,
            )))]),
        };

        // Outer if: v.tag == INT
        let outer_condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT".to_string())),
        };

        let stmt = HirStatement::If {
            condition: outer_condition,
            then_block: vec![],
            else_block: Some(vec![inner_if]),
        };

        let result = gen.transform_tag_check(&stmt);
        assert!(result.contains("Value::Int"));
        assert!(result.contains("Value::Float"));
    }

    // ========================================================================
    // capitalize_tag_value tests
    // ========================================================================

    #[test]
    fn test_capitalize_simple() {
        let gen = PatternGenerator::new();

        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("int".to_string())),
        };

        let stmt = HirStatement::If {
            condition,
            then_block: vec![],
            else_block: None,
        };

        let result = gen.transform_tag_check(&stmt);
        // "int" should become "Int"
        assert!(result.contains("Value::Int"));
    }

    #[test]
    fn test_capitalize_underscore_separated() {
        let gen = PatternGenerator::new();

        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("SOME_VALUE".to_string())),
        };

        let stmt = HirStatement::If {
            condition,
            then_block: vec![],
            else_block: None,
        };

        let result = gen.transform_tag_check(&stmt);
        // "SOME_VALUE" should become "SomeValue"
        assert!(result.contains("Value::SomeValue"));
    }

    #[test]
    fn test_capitalize_all_caps() {
        let gen = PatternGenerator::new();

        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("FLOAT".to_string())),
        };

        let stmt = HirStatement::If {
            condition,
            then_block: vec![],
            else_block: None,
        };

        let result = gen.transform_tag_check(&stmt);
        // "FLOAT" should become "Float"
        assert!(result.contains("Value::Float"));
    }

    // ========================================================================
    // Edge case tests
    // ========================================================================

    #[test]
    fn test_binary_op_left_not_field_access() {
        let gen = PatternGenerator::new();

        // Binary op where left side is not field access
        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::Variable("tag".to_string())),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT".to_string())),
        };

        let stmt = HirStatement::If {
            condition,
            then_block: vec![],
            else_block: None,
        };

        let result = gen.transform_tag_check(&stmt);
        assert!(result.is_empty());
    }

    #[test]
    fn test_field_access_object_not_variable() {
        let gen = PatternGenerator::new();

        // Field access where object is not a simple variable
        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::IntLiteral(0)),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT".to_string())),
        };

        let stmt = HirStatement::If {
            condition,
            then_block: vec![],
            else_block: None,
        };

        let result = gen.transform_tag_check(&stmt);
        assert!(result.is_empty());
    }

    #[test]
    fn test_right_side_not_variable() {
        let gen = PatternGenerator::new();

        // Right side is not a variable (literal)
        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::IntLiteral(0)),
        };

        let stmt = HirStatement::If {
            condition,
            then_block: vec![],
            else_block: None,
        };

        let result = gen.transform_tag_check(&stmt);
        assert!(result.is_empty());
    }

    #[test]
    fn test_then_block_with_expression_statement() {
        let gen = PatternGenerator::new();

        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT".to_string())),
        };

        // Then block with expression accessing v.data.i
        let union_field = HirExpression::FieldAccess {
            object: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "data".to_string(),
            }),
            field: "i".to_string(),
        };

        let stmt = HirStatement::If {
            condition,
            then_block: vec![HirStatement::Expression(union_field)],
            else_block: None,
        };

        let result = gen.transform_tag_check(&stmt);
        assert!(result.contains("Value::Int(i)"));
    }

    #[test]
    fn test_then_block_empty() {
        let gen = PatternGenerator::new();

        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT".to_string())),
        };

        let stmt = HirStatement::If {
            condition,
            then_block: vec![],
            else_block: None,
        };

        let result = gen.transform_tag_check(&stmt);
        assert!(result.contains("{}"));
    }

    #[test]
    fn test_then_block_multiple_statements() {
        let gen = PatternGenerator::new();

        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT".to_string())),
        };

        let stmt = HirStatement::If {
            condition,
            then_block: vec![HirStatement::Break, HirStatement::Continue],
            else_block: None,
        };

        let result = gen.transform_tag_check(&stmt);
        assert!(result.contains("{ /* block */ }"));
    }

    #[test]
    fn test_else_block_empty() {
        let gen = PatternGenerator::new();

        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT".to_string())),
        };

        let stmt = HirStatement::If {
            condition,
            then_block: vec![],
            else_block: Some(vec![]),
        };

        let result = gen.transform_tag_check(&stmt);
        assert!(result.contains("_ => {}"));
    }

    #[test]
    fn test_else_block_multiple_non_if_statements() {
        let gen = PatternGenerator::new();

        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT".to_string())),
        };

        let stmt = HirStatement::If {
            condition,
            then_block: vec![],
            else_block: Some(vec![HirStatement::Break, HirStatement::Continue]),
        };

        let result = gen.transform_tag_check(&stmt);
        assert!(result.contains("_ => { /* block */ }"));
    }

    #[test]
    fn test_union_field_not_data_field() {
        let gen = PatternGenerator::new();

        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT".to_string())),
        };

        // Return v.other.i (not v.data.i)
        let non_data_field = HirExpression::FieldAccess {
            object: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "other".to_string(),
            }),
            field: "i".to_string(),
        };

        let stmt = HirStatement::If {
            condition,
            then_block: vec![HirStatement::Return(Some(non_data_field))],
            else_block: None,
        };

        let result = gen.transform_tag_check(&stmt);
        // Should not extract field binding since it's not v.data.*
        assert!(result.contains("Value::Int =>"));
        assert!(!result.contains("Value::Int("));
    }

    #[test]
    fn test_return_without_union_access() {
        let gen = PatternGenerator::new();

        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT".to_string())),
        };

        // Return simple variable (not union field access)
        let stmt = HirStatement::If {
            condition,
            then_block: vec![HirStatement::Return(Some(HirExpression::Variable(
                "x".to_string(),
            )))],
            else_block: None,
        };

        let result = gen.transform_tag_check(&stmt);
        assert!(result.contains("return /* value */"));
    }

    #[test]
    fn test_capitalize_empty_string() {
        let gen = PatternGenerator::new();

        // Test with empty tag value - should use as-is
        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("".to_string())),
        };

        let stmt = HirStatement::If {
            condition,
            then_block: vec![],
            else_block: None,
        };

        let result = gen.transform_tag_check(&stmt);
        // Empty tag should be preserved
        assert!(result.contains("Value::"));
    }

    #[test]
    fn test_capitalize_multiple_underscores() {
        let gen = PatternGenerator::new();

        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("MY__TAG".to_string())),
        };

        let stmt = HirStatement::If {
            condition,
            then_block: vec![],
            else_block: None,
        };

        let result = gen.transform_tag_check(&stmt);
        // Double underscore should be handled - empty parts filtered
        assert!(result.contains("Value::MyTag"));
    }

    #[test]
    fn test_find_union_field_in_stmt_other_variants() {
        let gen = PatternGenerator::new();

        let condition = HirExpression::BinaryOp {
            left: Box::new(HirExpression::FieldAccess {
                object: Box::new(HirExpression::Variable("v".to_string())),
                field: "tag".to_string(),
            }),
            op: BinaryOperator::Equal,
            right: Box::new(HirExpression::Variable("INT".to_string())),
        };

        // Test with non-return, non-expression statement
        let stmt = HirStatement::If {
            condition,
            then_block: vec![HirStatement::Break],
            else_block: None,
        };

        let result = gen.transform_tag_check(&stmt);
        // Should still generate match but without field binding
        assert!(result.contains("Value::Int"));
    }
}
