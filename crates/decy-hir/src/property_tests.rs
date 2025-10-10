//! Property tests for HIR (DECY-002 REFACTOR phase).
//!
//! These tests verify HIR invariants using property-based testing.

use super::*;
use proptest::prelude::*;

// Strategy for generating HirType
fn hir_type_strategy() -> impl Strategy<Value = HirType> {
    prop_oneof![
        Just(HirType::Void),
        Just(HirType::Int),
        Just(HirType::Float),
        Just(HirType::Double),
        Just(HirType::Char),
        // Limit pointer depth to avoid stack overflow
        Just(HirType::Pointer(Box::new(HirType::Int))),
    ]
}

// Strategy for generating HirParameter
fn hir_parameter_strategy() -> impl Strategy<Value = HirParameter> {
    ("[a-z]{1,10}", hir_type_strategy()).prop_map(|(name, param_type)| {
        HirParameter::new(name, param_type)
    })
}

// Strategy for generating HirFunction
fn hir_function_strategy() -> impl Strategy<Value = HirFunction> {
    (
        "[a-z_][a-z0-9_]{0,20}",
        hir_type_strategy(),
        prop::collection::vec(hir_parameter_strategy(), 0..5),
    )
        .prop_map(|(name, return_type, parameters)| {
            HirFunction::new(name, return_type, parameters)
        })
}

proptest! {
    /// Property: HirFunction name is always accessible
    #[test]
    fn property_hir_function_name_accessible(func in hir_function_strategy()) {
        let name = func.name();
        prop_assert!(!name.is_empty());
    }

    /// Property: HirFunction return type is always accessible
    #[test]
    fn property_hir_function_return_type_accessible(func in hir_function_strategy()) {
        let _ = func.return_type();
    }

    /// Property: HirFunction parameters length matches input
    #[test]
    fn property_hir_function_parameters_count(
        name in "[a-z_][a-z0-9_]{0,20}",
        return_type in hir_type_strategy(),
        params in prop::collection::vec(hir_parameter_strategy(), 0..10)
    ) {
        let param_count = params.len();
        let func = HirFunction::new(name, return_type, params);
        prop_assert_eq!(func.parameters().len(), param_count);
    }

    /// Property: HirFunction is cloneable and equals itself
    #[test]
    fn property_hir_function_clone_equals(func in hir_function_strategy()) {
        let cloned = func.clone();
        prop_assert_eq!(func, cloned);
    }

    /// Property: HirParameter name is always accessible
    #[test]
    fn property_hir_parameter_name_accessible(param in hir_parameter_strategy()) {
        let name = param.name();
        prop_assert!(!name.is_empty());
    }

    /// Property: HirParameter type is always accessible
    #[test]
    fn property_hir_parameter_type_accessible(param in hir_parameter_strategy()) {
        let _ = param.param_type();
    }

    /// Property: HirParameter is cloneable and equals itself
    #[test]
    fn property_hir_parameter_clone_equals(param in hir_parameter_strategy()) {
        let cloned = param.clone();
        prop_assert_eq!(param, cloned);
    }

    /// Property: HirType is cloneable and equals itself
    #[test]
    fn property_hir_type_clone_equals(hir_type in hir_type_strategy()) {
        let cloned = hir_type.clone();
        prop_assert_eq!(hir_type, cloned);
    }

    /// Property: Pointer type always contains inner type
    #[test]
    fn property_pointer_has_inner_type(inner_type in hir_type_strategy()) {
        let ptr = HirType::Pointer(Box::new(inner_type.clone()));
        if let HirType::Pointer(inner) = ptr {
            prop_assert_eq!(*inner, inner_type);
        } else {
            prop_assert!(false, "Expected pointer type");
        }
    }

    /// Property: AST to HIR conversion preserves function name
    #[test]
    fn property_ast_to_hir_preserves_name(name in "[a-z_][a-z0-9_]{0,20}") {
        use decy_parser::parser::{Function, Type};

        let ast_func = Function::new(name.clone(), Type::Int, vec![]);
        let hir_func = HirFunction::from_ast_function(&ast_func);

        prop_assert_eq!(hir_func.name(), name.as_str());
    }

    /// Property: AST to HIR conversion preserves parameter count
    #[test]
    fn property_ast_to_hir_preserves_param_count(param_count in 0usize..10) {
        use decy_parser::parser::{Function, Type, Parameter};

        let params: Vec<_> = (0..param_count)
            .map(|i| Parameter::new(format!("p{}", i), Type::Int))
            .collect();

        let ast_func = Function::new("test".to_string(), Type::Void, params);
        let hir_func = HirFunction::from_ast_function(&ast_func);

        prop_assert_eq!(hir_func.parameters().len(), param_count);
    }
}
