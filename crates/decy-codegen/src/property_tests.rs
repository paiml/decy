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

    // DECY-005 property tests for if/else statements

    /// Property: If statements always contain "if" keyword
    #[test]
    fn property_if_contains_if_keyword(val in -100i32..100i32) {
        use decy_hir::{BinaryOperator, HirExpression, HirStatement};

        let if_stmt = HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::IntLiteral(val)),
            },
            then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
            else_block: None,
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&if_stmt);

        prop_assert!(code.starts_with("if "));
    }

    /// Property: If statements have balanced braces
    #[test]
    fn property_if_balanced_braces(val in -100i32..100i32) {
        use decy_hir::{BinaryOperator, HirExpression, HirStatement};

        let if_stmt = HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::IntLiteral(val)),
            },
            then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(0)))],
            else_block: None,
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&if_stmt);

        let open = code.matches('{').count();
        let close = code.matches('}').count();
        prop_assert_eq!(open, close);
        prop_assert!(open >= 1);
    }

    /// Property: If/else statements contain "else" keyword
    #[test]
    fn property_if_else_contains_else(val in -100i32..100i32) {
        use decy_hir::{BinaryOperator, HirExpression, HirStatement};

        let if_stmt = HirStatement::If {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::Equal,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::IntLiteral(val)),
            },
            then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
            else_block: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(-1)))]),
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&if_stmt);

        // Check for else keyword
        prop_assert!(code.contains("else"));
    }

    // DECY-006 property tests for while loops

    /// Property: While statements always contain "while" keyword
    #[test]
    fn property_while_contains_while_keyword(val in -100i32..100i32) {
        use decy_hir::{BinaryOperator, HirExpression, HirStatement};

        let while_stmt = HirStatement::While {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::IntLiteral(val)),
            },
            body: vec![HirStatement::Continue],
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&while_stmt);

        prop_assert!(code.starts_with("while "));
    }

    /// Property: While statements have balanced braces
    #[test]
    fn property_while_balanced_braces(val in -100i32..100i32) {
        use decy_hir::{BinaryOperator, HirExpression, HirStatement};

        let while_stmt = HirStatement::While {
            condition: HirExpression::BinaryOp {
                op: BinaryOperator::GreaterThan,
                left: Box::new(HirExpression::Variable("x".to_string())),
                right: Box::new(HirExpression::IntLiteral(val)),
            },
            body: vec![HirStatement::Break],
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&while_stmt);

        let open = code.matches('{').count();
        let close = code.matches('}').count();
        prop_assert_eq!(open, close);
        prop_assert!(open >= 1);
    }

    /// Property: Break statement always generates "break;"
    #[test]
    fn property_break_is_constant(_seed in 0u32..100) {
        use decy_hir::HirStatement;

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&HirStatement::Break);

        prop_assert_eq!(code, "break;");
    }

    /// Property: Continue statement always generates "continue;"
    #[test]
    fn property_continue_is_constant(_seed in 0u32..100) {
        use decy_hir::HirStatement;

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&HirStatement::Continue);

        prop_assert_eq!(code, "continue;");
    }

    // DECY-008 property tests for pointer operations

    /// Property: Dereference always starts with "*"
    #[test]
    fn property_dereference_starts_with_star(name in "[a-z_][a-z0-9_]{0,10}") {
        use decy_hir::HirExpression;

        let expr = HirExpression::Dereference(Box::new(HirExpression::Variable(name)));

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        prop_assert!(code.starts_with('*'));
    }

    /// Property: AddressOf always starts with "&"
    #[test]
    fn property_address_of_starts_with_ampersand(name in "[a-z_][a-z0-9_]{0,10}") {
        use decy_hir::HirExpression;

        let expr = HirExpression::AddressOf(Box::new(HirExpression::Variable(name)));

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        prop_assert!(code.starts_with('&'));
    }

    /// Property: Nested dereferences maintain proper star count
    #[test]
    fn property_nested_dereference_star_count(depth in 1usize..5) {
        use decy_hir::HirExpression;

        // Build nested dereferences: *, **, ***, etc.
        let mut expr = HirExpression::Variable("ptr".to_string());
        for _ in 0..depth {
            expr = HirExpression::Dereference(Box::new(expr));
        }

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        // Should have exactly 'depth' stars
        let star_count = code.chars().filter(|&c| c == '*').count();
        prop_assert_eq!(star_count, depth);
    }

    /// Property: Pointer operations generate consistent code
    #[test]
    fn property_pointer_ops_consistent(name in "[a-z_][a-z0-9_]{0,10}") {
        use decy_hir::HirExpression;

        let expr = HirExpression::Dereference(Box::new(HirExpression::Variable(name.clone())));

        let codegen = CodeGenerator::new();
        let code1 = codegen.generate_expression(&expr);
        let code2 = codegen.generate_expression(&expr);

        prop_assert_eq!(code1, code2);
    }

    // DECY-009 property tests for function calls

    /// Property: Function calls always contain parentheses
    #[test]
    fn property_function_call_has_parens(name in "[a-z_][a-z0-9_]{0,10}") {
        use decy_hir::HirExpression;

        let expr = HirExpression::FunctionCall {
            function: name,
            arguments: vec![],
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        prop_assert!(code.contains('('));
        prop_assert!(code.contains(')'));
    }

    /// Property: Function call preserves function name
    #[test]
    fn property_function_call_preserves_name(name in "[a-z_][a-z0-9_]{0,10}") {
        use decy_hir::HirExpression;

        let expr = HirExpression::FunctionCall {
            function: name.clone(),
            arguments: vec![],
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        prop_assert!(code.starts_with(&name));
    }

    /// Property: Function calls are deterministic
    #[test]
    fn property_function_call_consistent(
        name in "[a-z_][a-z0-9_]{0,10}",
        arg_count in 0usize..5
    ) {
        use decy_hir::HirExpression;

        let args = (0..arg_count)
            .map(|i| HirExpression::IntLiteral(i as i32))
            .collect::<Vec<_>>();

        let expr = HirExpression::FunctionCall {
            function: name,
            arguments: args,
        };

        let codegen = CodeGenerator::new();
        let code1 = codegen.generate_expression(&expr);
        let code2 = codegen.generate_expression(&expr);

        prop_assert_eq!(code1, code2);
    }

    /// Property: Function call with N args has N-1 commas
    #[test]
    fn property_function_call_comma_count(arg_count in 1usize..6) {
        use decy_hir::HirExpression;

        let args = (0..arg_count)
            .map(|i| HirExpression::IntLiteral(i as i32))
            .collect::<Vec<_>>();

        let expr = HirExpression::FunctionCall {
            function: "foo".to_string(),
            arguments: args,
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        let comma_count = code.matches(',').count();
        prop_assert_eq!(comma_count, arg_count - 1);
    }

    // DECY-009 Phase 2: property tests for assignment statements

    /// Property: Assignment statements always contain "="
    #[test]
    fn property_assignment_contains_equals(
        target in "[a-z_][a-z0-9_]{0,10}",
        val in -100i32..100i32
    ) {
        use decy_hir::{HirExpression, HirStatement};

        let assign_stmt = HirStatement::Assignment {
            target,
            value: HirExpression::IntLiteral(val),
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&assign_stmt);

        prop_assert!(code.contains(" = "));
    }

    /// Property: Assignment statements end with semicolon
    #[test]
    fn property_assignment_ends_with_semicolon(
        target in "[a-z_][a-z0-9_]{0,10}",
        val in -100i32..100i32
    ) {
        use decy_hir::{HirExpression, HirStatement};

        let assign_stmt = HirStatement::Assignment {
            target,
            value: HirExpression::IntLiteral(val),
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&assign_stmt);

        prop_assert!(code.ends_with(';'));
    }

    /// Property: Assignment preserves target variable name
    #[test]
    fn property_assignment_preserves_target(
        target in "[a-z_][a-z0-9_]{0,10}",
        val in -100i32..100i32
    ) {
        use decy_hir::{HirExpression, HirStatement};

        let assign_stmt = HirStatement::Assignment {
            target: target.clone(),
            value: HirExpression::IntLiteral(val),
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_statement(&assign_stmt);

        prop_assert!(code.starts_with(&target));
    }

    /// Property: Assignment code generation is deterministic
    #[test]
    fn property_assignment_consistent(
        target in "[a-z_][a-z0-9_]{0,10}",
        val in -100i32..100i32
    ) {
        use decy_hir::{HirExpression, HirStatement};

        let assign_stmt = HirStatement::Assignment {
            target,
            value: HirExpression::IntLiteral(val),
        };

        let codegen = CodeGenerator::new();
        let code1 = codegen.generate_statement(&assign_stmt);
        let code2 = codegen.generate_statement(&assign_stmt);

        prop_assert_eq!(code1, code2);
    }

    // DECY-044 property tests for sizeof operator

    /// Property: Sizeof always contains "size_of"
    #[test]
    fn property_sizeof_contains_size_of(
        type_name in prop_oneof![
            Just("int".to_string()),
            Just("float".to_string()),
            Just("double".to_string()),
            Just("char".to_string()),
            Just("struct Data".to_string()),
        ]
    ) {
        use decy_hir::HirExpression;

        let expr = HirExpression::Sizeof {
            type_name,
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        prop_assert!(code.contains("size_of"));
    }

    /// Property: Sizeof always contains "std::mem"
    #[test]
    fn property_sizeof_contains_std_mem(
        type_name in prop_oneof![
            Just("int".to_string()),
            Just("float".to_string()),
            Just("double".to_string()),
        ]
    ) {
        use decy_hir::HirExpression;

        let expr = HirExpression::Sizeof {
            type_name,
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        prop_assert!(code.contains("std::mem"));
    }

    /// Property: Sizeof always casts to i32
    #[test]
    fn property_sizeof_casts_to_i32(
        type_name in prop_oneof![
            Just("int".to_string()),
            Just("char".to_string()),
            Just("struct Point".to_string()),
        ]
    ) {
        use decy_hir::HirExpression;

        let expr = HirExpression::Sizeof {
            type_name,
        };

        let codegen = CodeGenerator::new();
        let code = codegen.generate_expression(&expr);

        prop_assert!(code.contains("as i32"));
    }

    /// Property: Sizeof code generation is deterministic
    #[test]
    fn property_sizeof_consistent(
        type_name in prop_oneof![
            Just("int".to_string()),
            Just("float".to_string()),
            Just("struct Data".to_string()),
        ]
    ) {
        use decy_hir::HirExpression;

        let expr = HirExpression::Sizeof {
            type_name,
        };

        let codegen = CodeGenerator::new();
        let code1 = codegen.generate_expression(&expr);
        let code2 = codegen.generate_expression(&expr);

        prop_assert_eq!(code1, code2);
    }

    /// Property: Sizeof type mapping is consistent
    #[test]
    fn property_sizeof_type_mapping_consistent(
        type_name in prop_oneof![
            Just("int".to_string()),
            Just("float".to_string()),
            Just("double".to_string()),
            Just("char".to_string()),
        ]
    ) {
        let codegen = CodeGenerator::new();
        let first = codegen.map_sizeof_type(&type_name);
        let second = codegen.map_sizeof_type(&type_name);

        prop_assert_eq!(first, second);
    }

    /// Property: Sizeof strips "struct " prefix for struct types
    #[test]
    fn property_sizeof_strips_struct_prefix(
        struct_name in "[A-Z][a-zA-Z0-9_]{0,10}"
    ) {
        let type_name = format!("struct {}", struct_name);
        let codegen = CodeGenerator::new();
        let mapped = codegen.map_sizeof_type(&type_name);

        prop_assert!(!mapped.contains("struct"));
        prop_assert_eq!(mapped, struct_name);
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
