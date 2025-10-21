//! Property-based tests for Vec code generation (DECY-019 TDD-Refactor phase).
//!
//! Tests ensure that Vec generation is deterministic, never panics,
//! and maintains invariants across random inputs.

use super::*;
use decy_analyzer::patterns::VecCandidate;
use decy_hir::{BinaryOperator, HirExpression, HirFunction, HirStatement, HirType};
use proptest::prelude::*;

proptest! {
    /// Property: Vec transformation never panics on valid inputs
    #[test]
    fn property_vec_transform_never_panics(
        var_name in "[a-z_][a-z0-9_]{0,10}",
        capacity in 1i32..1000,
        element_size in 1i32..16
    ) {
        let codegen = CodeGenerator::new();
        let candidate = VecCandidate {
            variable: var_name.clone(),
            malloc_index: 0,
            free_index: None,
            capacity_expr: Some(HirExpression::IntLiteral(capacity)),
        };

        let size_expr = HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::IntLiteral(capacity)),
            right: Box::new(HirExpression::IntLiteral(element_size)),
        };

        let stmt = HirStatement::VariableDeclaration {
            name: var_name,
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![size_expr],
            }),
        };

        let _transformed = codegen.transform_vec_statement(&stmt, &candidate);
        // If we get here without panic, test passes
    }

    /// Property: Transformed malloc always becomes Vec::with_capacity or Vec::new
    #[test]
    fn property_malloc_becomes_vec(
        capacity in 1i32..1000
    ) {
        let codegen = CodeGenerator::new();
        let candidate = VecCandidate {
            variable: "arr".to_string(),
            malloc_index: 0,
            free_index: None,
            capacity_expr: Some(HirExpression::IntLiteral(capacity)),
        };

        let size_expr = HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::IntLiteral(capacity)),
            right: Box::new(HirExpression::IntLiteral(4)),
        };

        let stmt = HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![size_expr],
            }),
        };

        let transformed = codegen.transform_vec_statement(&stmt, &candidate);

        match transformed {
            HirStatement::VariableDeclaration { initializer: Some(expr), .. } => {
                match expr {
                    HirExpression::FunctionCall { function, .. } => {
                        prop_assert!(
                            function == "Vec::with_capacity" || function == "Vec::new",
                            "Expected Vec::with_capacity or Vec::new, got {}",
                            function
                        );
                    }
                    _ => prop_assert!(false, "Expected FunctionCall"),
                }
            }
            _ => prop_assert!(false, "Expected VariableDeclaration"),
        }
    }

    /// Property: Vec::with_capacity always has exactly one argument
    #[test]
    fn property_vec_with_capacity_has_one_arg(
        capacity in 1i32..1000
    ) {
        let codegen = CodeGenerator::new();
        let candidate = VecCandidate {
            variable: "arr".to_string(),
            malloc_index: 0,
            free_index: None,
            capacity_expr: Some(HirExpression::IntLiteral(capacity)),
        };

        let size_expr = HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::IntLiteral(capacity)),
            right: Box::new(HirExpression::IntLiteral(4)),
        };

        let stmt = HirStatement::VariableDeclaration {
            name: "arr".to_string(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![size_expr],
            }),
        };

        let transformed = codegen.transform_vec_statement(&stmt, &candidate);

        match transformed {
            HirStatement::VariableDeclaration { initializer: Some(expr), .. } => {
                match expr {
                    HirExpression::FunctionCall { function, arguments } => {
                        if function == "Vec::with_capacity" {
                            prop_assert_eq!(arguments.len(), 1);
                        } else if function == "Vec::new" {
                            prop_assert_eq!(arguments.len(), 0);
                        }
                    }
                    _ => prop_assert!(false, "Expected FunctionCall"),
                }
            }
            _ => prop_assert!(false, "Expected VariableDeclaration"),
        }
    }

    /// Property: Transform preserves variable name
    #[test]
    fn property_transform_preserves_name(
        var_name in "[a-z_][a-z0-9_]{0,10}",
        capacity in 1i32..1000
    ) {
        let codegen = CodeGenerator::new();
        let candidate = VecCandidate {
            variable: var_name.clone(),
            malloc_index: 0,
            free_index: None,
            capacity_expr: Some(HirExpression::IntLiteral(capacity)),
        };

        let size_expr = HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::IntLiteral(capacity)),
            right: Box::new(HirExpression::IntLiteral(4)),
        };

        let stmt = HirStatement::VariableDeclaration {
            name: var_name.clone(),
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![size_expr],
            }),
        };

        let transformed = codegen.transform_vec_statement(&stmt, &candidate);

        match transformed {
            HirStatement::VariableDeclaration { name, .. } => {
                prop_assert_eq!(&name, &var_name);
            }
            _ => prop_assert!(false, "Expected VariableDeclaration"),
        }
    }

    /// Property: Transform generates Vec type (not pointer type)
    #[test]
    fn property_transform_generates_vec_type(
        var_name in "[a-z_][a-z0-9_]{0,10}",
        capacity in 1i32..1000
    ) {
        let codegen = CodeGenerator::new();
        let candidate = VecCandidate {
            variable: var_name.clone(),
            malloc_index: 0,
            free_index: None,
            capacity_expr: Some(HirExpression::IntLiteral(capacity)),
        };

        let size_expr = HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::IntLiteral(capacity)),
            right: Box::new(HirExpression::IntLiteral(4)),
        };

        let stmt = HirStatement::VariableDeclaration {
            name: var_name,
            var_type: HirType::Pointer(Box::new(HirType::Int)),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![size_expr],
            }),
        };

        let transformed = codegen.transform_vec_statement(&stmt, &candidate);

        match transformed {
            HirStatement::VariableDeclaration { var_type, .. } => {
                prop_assert!(matches!(var_type, HirType::Vec(_)), "Expected Vec type, got {:?}", var_type);
            }
            _ => prop_assert!(false, "Expected VariableDeclaration"),
        }
    }

    /// Property: Vec element type matches pointer element type
    #[test]
    fn property_vec_element_type_matches_pointer(
        var_name in "[a-z_][a-z0-9_]{0,10}",
        capacity in 1i32..1000,
        element_type_idx in 0usize..4
    ) {
        let element_types = [HirType::Int, HirType::Char, HirType::Float, HirType::Double];
        let element_type = element_types[element_type_idx].clone();

        let codegen = CodeGenerator::new();
        let candidate = VecCandidate {
            variable: var_name.clone(),
            malloc_index: 0,
            free_index: None,
            capacity_expr: Some(HirExpression::IntLiteral(capacity)),
        };

        let size_expr = HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::IntLiteral(capacity)),
            right: Box::new(HirExpression::IntLiteral(4)),
        };

        let stmt = HirStatement::VariableDeclaration {
            name: var_name,
            var_type: HirType::Pointer(Box::new(element_type.clone())),
            initializer: Some(HirExpression::FunctionCall {
                function: "malloc".to_string(),
                arguments: vec![size_expr],
            }),
        };

        let transformed = codegen.transform_vec_statement(&stmt, &candidate);

        match transformed {
            HirStatement::VariableDeclaration { var_type: HirType::Vec(inner), .. } => {
                prop_assert_eq!(*inner, element_type);
            }
            _ => prop_assert!(false, "Expected VariableDeclaration with Vec type"),
        }
    }

    /// Property: Full code generation is deterministic
    #[test]
    fn property_code_generation_deterministic(
        var_name in "[a-z_][a-z0-9_]{0,10}",
        capacity in 1i32..100
    ) {
        let codegen = CodeGenerator::new();

        let size_expr = HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::IntLiteral(capacity)),
            right: Box::new(HirExpression::IntLiteral(4)),
        };

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![HirStatement::VariableDeclaration {
                name: var_name.clone(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![size_expr.clone()],
                }),
            }],
        );

        let candidate = VecCandidate {
            variable: var_name,
            malloc_index: 0,
            free_index: None,
            capacity_expr: Some(HirExpression::IntLiteral(capacity)),
        };

        let candidates = [candidate];
        let code1 = codegen.generate_function_with_vec_transform(&func, &candidates);
        let code2 = codegen.generate_function_with_vec_transform(&func, &candidates);

        prop_assert_eq!(code1, code2, "Vec code generation should be deterministic");
    }

    /// Property: Generated code contains Vec syntax
    #[test]
    fn property_generated_code_contains_vec(
        capacity in 1i32..100
    ) {
        let codegen = CodeGenerator::new();

        let size_expr = HirExpression::BinaryOp {
            op: BinaryOperator::Multiply,
            left: Box::new(HirExpression::IntLiteral(capacity)),
            right: Box::new(HirExpression::IntLiteral(4)),
        };

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            vec![],
            vec![HirStatement::VariableDeclaration {
                name: "arr".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::FunctionCall {
                    function: "malloc".to_string(),
                    arguments: vec![size_expr],
                }),
            }],
        );

        let candidate = VecCandidate {
            variable: "arr".to_string(),
            malloc_index: 0,
            free_index: None,
            capacity_expr: Some(HirExpression::IntLiteral(capacity)),
        };

        let code = codegen.generate_function_with_vec_transform(&func, &[candidate]);

        prop_assert!(code.contains("Vec<"), "Generated code should contain Vec< syntax");
        prop_assert!(!code.contains("*mut"), "Generated code should not contain *mut syntax");
    }
}
