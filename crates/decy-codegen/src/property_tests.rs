//! Property tests for code generator (DECY-003 REFACTOR phase).

use super::*;
use decy_hir::{HirFunction, HirType};
use proptest::prelude::*;

// Strategy for generating HIR types (reuse from decy-hir concepts)
fn hir_type_strategy() -> impl Strategy<Value = HirType> {
    prop_oneof![
        Just(HirType::Void),
        Just(HirType::Int),
        Just(HirType::Float),
        Just(HirType::Double),
        Just(HirType::Char),
        Just(HirType::Pointer(Box::new(HirType::Int))),
    ]
}

proptest! {
    /// Property: Generated code always contains "fn" keyword
    #[test]
    fn property_generated_code_has_fn_keyword(
        name in "[a-z_][a-z0-9_]{0,20}",
        return_type in hir_type_strategy()
    ) {
        let func = HirFunction::new(name, return_type, vec![]);
        let codegen = CodeGenerator::new();
        let code = codegen.generate_function(&func);

        prop_assert!(code.contains("fn "));
    }

    /// Property: Generated code has balanced braces
    #[test]
    fn property_generated_code_balanced_braces(
        name in "[a-z_][a-z0-9_]{0,20}",
        return_type in hir_type_strategy()
    ) {
        let func = HirFunction::new(name, return_type, vec![]);
        let codegen = CodeGenerator::new();
        let code = codegen.generate_function(&func);

        let open = code.matches('{').count();
        let close = code.matches('}').count();
        prop_assert_eq!(open, close);
        prop_assert!(open > 0);
    }

    /// Property: Type mapping is consistent
    #[test]
    fn property_type_mapping_consistent(hir_type in hir_type_strategy()) {
        let first = CodeGenerator::map_type(&hir_type);
        let second = CodeGenerator::map_type(&hir_type);

        prop_assert_eq!(first, second);
    }

    /// Property: Void functions have no return type annotation
    #[test]
    fn property_void_functions_no_return_annotation(
        name in "[a-z_][a-z0-9_]{0,20}"
    ) {
        let func = HirFunction::new(name.clone(), HirType::Void, vec![]);
        let codegen = CodeGenerator::new();
        let sig = codegen.generate_signature(&func);

        prop_assert!(!sig.contains("->"));
        prop_assert!(sig.contains(&name));
    }

    /// Property: Non-void functions have return type annotation
    #[test]
    fn property_non_void_has_return_annotation(
        name in "[a-z_][a-z0-9_]{0,20}",
        return_type in prop_oneof![
            Just(HirType::Int),
            Just(HirType::Float),
            Just(HirType::Double),
        ]
    ) {
        let func = HirFunction::new(name, return_type, vec![]);
        let codegen = CodeGenerator::new();
        let sig = codegen.generate_signature(&func);

        prop_assert!(sig.contains("->"));
    }

    /// Property: Generated signature contains function name
    #[test]
    fn property_signature_contains_name(
        name in "[a-z_][a-z0-9_]{0,20}",
        return_type in hir_type_strategy()
    ) {
        let func = HirFunction::new(name.clone(), return_type, vec![]);
        let codegen = CodeGenerator::new();
        let sig = codegen.generate_signature(&func);

        prop_assert!(sig.contains(&name));
    }

    // DECY-004 property tests for statements

    /// Property: Generated variable declarations always start with "let mut"
    #[test]
    fn property_all_vars_are_mutable(
        name in "[a-z_][a-z0-9_]{0,10}",
        var_type in hir_type_strategy()
    ) {
        use decy_hir::{HirStatement};

        let var_decl = HirStatement::VariableDeclaration {
            name,
            var_type,
            initializer: None,
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&var_decl);

        prop_assert!(code.starts_with("let mut"));
    }

    /// Property: Int literals are rendered as numbers
    #[test]
    fn property_int_literal_is_numeric(val in -1000i32..1000i32) {
        use decy_hir::HirExpression;

        let expr = HirExpression::IntLiteral(val);

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        // Should be parseable as i32
        prop_assert_eq!(code.parse::<i32>().ok(), Some(val));
    }

    /// Property: Variable references preserve the name exactly
    #[test]
    fn property_var_ref_preserves_name(name in "[a-z_][a-z0-9_]{0,10}") {
        use decy_hir::HirExpression;

        let expr = HirExpression::Variable(name.clone());

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        prop_assert_eq!(code, name);
    }

    // DECY-007 property tests for binary expressions

    /// Property: Binary expressions with same operator generate consistent code
    #[test]
    fn property_binary_expr_consistent(
        val1 in -100i32..100i32,
        val2 in -100i32..100i32
    ) {
        use decy_hir::{BinaryOperator, HirExpression};

        let expr1 = HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::IntLiteral(val1)),
            right: Box::new(HirExpression::IntLiteral(val2)),
        };

        let expr2 = HirExpression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(HirExpression::IntLiteral(val1)),
            right: Box::new(HirExpression::IntLiteral(val2)),
        };

        let codegen = CodeGenerator::new();
        let code1 = codegen.generate_expression(&expr1);
        let code2 = codegen.generate_expression(&expr2);

        prop_assert_eq!(code1, code2);
    }
}

// Regular tests for binary expressions (not property tests)
#[cfg(test)]
mod binary_expr_tests {
    use super::*;
    use decy_hir::{BinaryOperator, HirExpression};

    #[test]
    fn test_binary_expr_contains_operator() {
        let ops = [
            (BinaryOperator::Add, "+"),
            (BinaryOperator::Subtract, "-"),
            (BinaryOperator::Multiply, "*"),
            (BinaryOperator::Divide, "/"),
            (BinaryOperator::Modulo, "%"),
            (BinaryOperator::Equal, "=="),
            (BinaryOperator::NotEqual, "!="),
            (BinaryOperator::LessThan, "<"),
            (BinaryOperator::GreaterThan, ">"),
            (BinaryOperator::LessEqual, "<="),
            (BinaryOperator::GreaterEqual, ">="),
        ];

        for (op, op_str) in ops {
            let expr = HirExpression::BinaryOp {
                op,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            };

            let codegen = CodeGenerator::new();
            let code = codegen.generate_expression(&expr);

            assert!(code.contains(op_str));
        }
    }

    #[test]
    fn test_nested_expr_has_parens() {
        // (a + b) * c
        let expr = HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("a".to_string())),
                right: Box::new(HirExpression::Variable("b".to_string())),
            }),
            right: Box::new(HirExpression::Variable("c".to_string())),
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        assert!(code.contains('('));
        assert!(code.contains(')'));
    }
}
