//! Property tests for switch/case code generation (DECY-026 TDD-Refactor phase).

use super::*;
use decy_hir::{BinaryOperator, HirExpression, HirStatement, SwitchCase};
use proptest::prelude::*;

proptest! {
    /// Property: Generated switch statements always contain "match"
    #[test]
    fn property_switch_contains_match(var_name in "[a-z_][a-z0-9_]{0,10}") {
        let switch_stmt = HirStatement::Switch {
            condition: HirExpression::Variable(var_name),
            cases: vec![
                SwitchCase {
                    value: Some(HirExpression::IntLiteral(1)),
                    body: vec![HirStatement::Break],
                },
            ],
            default_case: None,
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&switch_stmt);

        prop_assert!(code.contains("match"));
    }

    /// Property: Switch statements always have default case (_)
    #[test]
    fn property_switch_has_default(
        var_name in "[a-z_][a-z0-9_]{0,10}",
        case_val in 0i32..100
    ) {
        let switch_stmt = HirStatement::Switch {
            condition: HirExpression::Variable(var_name),
            cases: vec![
                SwitchCase {
                    value: Some(HirExpression::IntLiteral(case_val)),
                    body: vec![HirStatement::Break],
                },
            ],
            default_case: None,
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&switch_stmt);

        // Rust requires exhaustive matching, so _ => must be present
        prop_assert!(code.contains("_ =>"));
    }

    /// Property: Break statements are removed from generated code
    #[test]
    fn property_switch_removes_break(
        var_name in "[a-z_][a-z0-9_]{0,10}",
        case_val in 0i32..100
    ) {
        let switch_stmt = HirStatement::Switch {
            condition: HirExpression::Variable(var_name),
            cases: vec![
                SwitchCase {
                    value: Some(HirExpression::IntLiteral(case_val)),
                    body: vec![HirStatement::Break],
                },
            ],
            default_case: Some(vec![HirStatement::Break]),
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&switch_stmt);

        // Break statements should not appear in Rust match
        prop_assert!(!code.contains("break"));
    }

    /// Property: Code generation is deterministic
    #[test]
    fn property_switch_generation_deterministic(
        var_name in "[a-z_][a-z0-9_]{0,10}",
        case_val in 0i32..100
    ) {
        let switch_stmt = HirStatement::Switch {
            condition: HirExpression::Variable(var_name),
            cases: vec![
                SwitchCase {
                    value: Some(HirExpression::IntLiteral(case_val)),
                    body: vec![HirStatement::Break],
                },
            ],
            default_case: None,
        };

        let codegen = CodeGenerator::new();
        let code1 = codegen.generate_statement(&switch_stmt);
        let code2 = codegen.generate_statement(&switch_stmt);

        prop_assert_eq!(code1, code2);
    }

    /// Property: Each case value appears in generated code
    #[test]
    fn property_switch_preserves_case_values(
        var_name in "[a-z_][a-z0-9_]{0,10}",
        case_vals in prop::collection::vec(0i32..100, 1..5)
    ) {
        let cases: Vec<SwitchCase> = case_vals.iter().map(|&val| {
            SwitchCase {
                value: Some(HirExpression::IntLiteral(val)),
                body: vec![HirStatement::Break],
            }
        }).collect();

        let switch_stmt = HirStatement::Switch {
            condition: HirExpression::Variable(var_name),
            cases,
            default_case: None,
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&switch_stmt);

        // Each case value should appear in generated code
        for val in case_vals {
            let pattern = format!("{} =>", val);
            prop_assert!(code.contains(&pattern));
        }
    }

    /// Property: Condition variable appears in match
    #[test]
    fn property_switch_preserves_condition(
        var_name in "[a-z_][a-z0-9_]{0,10}"
    ) {
        let switch_stmt = HirStatement::Switch {
            condition: HirExpression::Variable(var_name.clone()),
            cases: vec![
                SwitchCase {
                    value: Some(HirExpression::IntLiteral(1)),
                    body: vec![HirStatement::Break],
                },
            ],
            default_case: None,
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&switch_stmt);

        prop_assert!(code.contains(&var_name));
    }

    /// Property: Switch with return statements preserves return values
    #[test]
    fn property_switch_preserves_return_values(
        var_name in "[a-z_][a-z0-9_]{0,10}",
        return_val in 0i32..100
    ) {
        let switch_stmt = HirStatement::Switch {
            condition: HirExpression::Variable(var_name),
            cases: vec![
                SwitchCase {
                    value: Some(HirExpression::IntLiteral(1)),
                    body: vec![HirStatement::Return(Some(HirExpression::IntLiteral(return_val)))],
                },
            ],
            default_case: None,
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&switch_stmt);

        let expected = format!("return {}", return_val);
        prop_assert!(code.contains(&expected));
    }

    /// Property: Generated code has balanced braces
    #[test]
    fn property_switch_balanced_braces(
        var_name in "[a-z_][a-z0-9_]{0,10}",
        num_cases in 1usize..5
    ) {
        let cases: Vec<SwitchCase> = (0..num_cases).map(|i| {
            SwitchCase {
                value: Some(HirExpression::IntLiteral(i as i32)),
                body: vec![HirStatement::Break],
            }
        }).collect();

        let switch_stmt = HirStatement::Switch {
            condition: HirExpression::Variable(var_name),
            cases,
            default_case: None,
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&switch_stmt);

        let open_braces = code.chars().filter(|&c| c == '{').count();
        let close_braces = code.chars().filter(|&c| c == '}').count();

        prop_assert_eq!(open_braces, close_braces);
    }

    /// Property: Switch with complex expression conditions preserves expression
    #[test]
    fn property_switch_preserves_expression_condition(
        var1 in "[a-z_][a-z0-9_]{0,10}",
        var2 in "[a-z_][a-z0-9_]{0,10}"
    ) {
        let switch_stmt = HirStatement::Switch {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable(var1.clone())),
                right: Box::new(HirExpression::Variable(var2.clone())),
            },
            cases: vec![
                SwitchCase {
                    value: Some(HirExpression::IntLiteral(1)),
                    body: vec![HirStatement::Break],
                },
            ],
            default_case: None,
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&switch_stmt);

        // Both variables should appear in the condition
        prop_assert!(code.contains(&var1));
        prop_assert!(code.contains(&var2));
    }
}
