//! Property tests for function pointer HIR representation (DECY-024 REFACTOR phase)
//!
//! This test suite uses proptest to verify function pointer type properties.
//! Target: 10 properties × 256 cases = 2,560 test cases.
//!
//! References:
//! - K&R §5.11: Pointers to Functions
//! - ISO C99 §6.7.5.3: Function declarators

use decy_hir::{HirParameter, HirType};
use proptest::prelude::*;

/// Generate simple HirType variants
fn simple_hir_type() -> impl Strategy<Value = HirType> {
    prop_oneof![
        Just(HirType::Int),
        Just(HirType::Float),
        Just(HirType::Double),
        Just(HirType::Char),
        Just(HirType::Void),
    ]
}

/// Generate valid identifier strings
fn valid_identifier() -> impl Strategy<Value = String> {
    "[a-zA-Z_][a-zA-Z0-9_]{0,30}".prop_map(|s| s.to_string())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property 1: Creating function pointer type should never panic
    #[test]
    fn prop_function_pointer_creation_never_panics(
        param_type in simple_hir_type(),
        return_type in simple_hir_type(),
    ) {
        let _ = HirType::FunctionPointer {
            param_types: vec![param_type],
            return_type: Box::new(return_type),
        };
    }

    /// Property 2: Function pointer with same types should be equal
    #[test]
    fn prop_function_pointer_equality(
        param_type in simple_hir_type(),
        return_type in simple_hir_type(),
    ) {
        let fp1 = HirType::FunctionPointer {
            param_types: vec![param_type.clone()],
            return_type: Box::new(return_type.clone()),
        };

        let fp2 = HirType::FunctionPointer {
            param_types: vec![param_type],
            return_type: Box::new(return_type),
        };

        prop_assert_eq!(fp1, fp2);
    }

    /// Property 3: Different param types should not be equal
    #[test]
    fn prop_function_pointer_inequality_params(
        param1 in simple_hir_type(),
        param2 in simple_hir_type(),
        return_type in simple_hir_type(),
    ) {
        prop_assume!(param1 != param2);

        let fp1 = HirType::FunctionPointer {
            param_types: vec![param1],
            return_type: Box::new(return_type.clone()),
        };

        let fp2 = HirType::FunctionPointer {
            param_types: vec![param2],
            return_type: Box::new(return_type),
        };

        prop_assert_ne!(fp1, fp2);
    }

    /// Property 4: Different return types should not be equal
    #[test]
    fn prop_function_pointer_inequality_return(
        param_type in simple_hir_type(),
        return1 in simple_hir_type(),
        return2 in simple_hir_type(),
    ) {
        prop_assume!(return1 != return2);

        let fp1 = HirType::FunctionPointer {
            param_types: vec![param_type.clone()],
            return_type: Box::new(return1),
        };

        let fp2 = HirType::FunctionPointer {
            param_types: vec![param_type],
            return_type: Box::new(return2),
        };

        prop_assert_ne!(fp1, fp2);
    }

    /// Property 5: Parameter count should be preserved
    #[test]
    fn prop_function_pointer_preserves_param_count(
        param_count in 0usize..5,
        return_type in simple_hir_type(),
    ) {
        let param_types = vec![HirType::Int; param_count];

        let fp = HirType::FunctionPointer {
            param_types: param_types.clone(),
            return_type: Box::new(return_type),
        };

        match fp {
            HirType::FunctionPointer { param_types: params, .. } => {
                prop_assert_eq!(params.len(), param_count);
            }
            _ => prop_assert!(false, "Expected FunctionPointer type"),
        }
    }

    /// Property 6: Return type should be preserved
    #[test]
    fn prop_function_pointer_preserves_return_type(
        return_type in simple_hir_type(),
    ) {
        let fp = HirType::FunctionPointer {
            param_types: vec![HirType::Int],
            return_type: Box::new(return_type.clone()),
        };

        match fp {
            HirType::FunctionPointer { return_type: ret, .. } => {
                prop_assert_eq!(*ret, return_type);
            }
            _ => prop_assert!(false, "Expected FunctionPointer type"),
        }
    }

    /// Property 7: Function pointer in parameter should work
    #[test]
    fn prop_function_pointer_as_parameter(
        param_name in valid_identifier(),
        return_type in simple_hir_type(),
    ) {
        let fp_type = HirType::FunctionPointer {
            param_types: vec![HirType::Int],
            return_type: Box::new(return_type),
        };

        let param = HirParameter::new(param_name.clone(), fp_type.clone());

        prop_assert_eq!(param.name(), param_name.as_str());
        prop_assert_eq!(param.param_type(), &fp_type);
    }

    /// Property 8: Void return should work
    #[test]
    fn prop_function_pointer_void_return(
        param_type in simple_hir_type(),
    ) {
        let fp = HirType::FunctionPointer {
            param_types: vec![param_type],
            return_type: Box::new(HirType::Void),
        };

        match fp {
            HirType::FunctionPointer { return_type, .. } => {
                prop_assert_eq!(*return_type, HirType::Void);
            }
            _ => prop_assert!(false, "Expected FunctionPointer type"),
        }
    }

    /// Property 9: Empty parameter list should work
    #[test]
    fn prop_function_pointer_no_params(
        return_type in simple_hir_type(),
    ) {
        let fp = HirType::FunctionPointer {
            param_types: vec![],
            return_type: Box::new(return_type),
        };

        match fp {
            HirType::FunctionPointer { param_types, .. } => {
                prop_assert_eq!(param_types.len(), 0);
            }
            _ => prop_assert!(false, "Expected FunctionPointer type"),
        }
    }

    /// Property 10: Cloning should preserve structure
    #[test]
    fn prop_function_pointer_clone_preserves(
        param_type in simple_hir_type(),
        return_type in simple_hir_type(),
    ) {
        let fp1 = HirType::FunctionPointer {
            param_types: vec![param_type],
            return_type: Box::new(return_type),
        };

        let fp2 = fp1.clone();

        prop_assert_eq!(fp1, fp2);
    }
}
