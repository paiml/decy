//! Property tests for HirTypedef representation (DECY-023 REFACTOR phase)
//!
//! This test suite uses proptest to verify HirTypedef properties.
//! Target: 10 properties × 256 cases = 2,560 test cases.
//!
//! References:
//! - K&R §6.7: Type Names
//! - ISO C99 §6.7.7: Type definitions

use decy_hir::{HirType, HirTypedef};
use proptest::prelude::*;

/// Generate valid Rust identifier strings (typedef names)
fn valid_identifier() -> impl Strategy<Value = String> {
    "[a-zA-Z_][a-zA-Z0-9_]{0,30}".prop_map(|s| s.to_string())
}

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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property 1: Creating HirTypedef should never panic
    #[test]
    fn prop_hir_typedef_creation_never_panics(
        name in valid_identifier(),
        base_type in simple_hir_type()
    ) {
        let _ = HirTypedef::new(name, base_type);
    }

    /// Property 2: Typedef name should always be preserved exactly
    #[test]
    fn prop_hir_typedef_name_preserved(
        name in valid_identifier(),
        base_type in simple_hir_type()
    ) {
        let typedef = HirTypedef::new(name.clone(), base_type);
        prop_assert_eq!(typedef.name(), name.as_str());
    }

    /// Property 3: Underlying type should match what was provided
    #[test]
    fn prop_hir_typedef_type_matches(
        name in valid_identifier(),
        base_type in simple_hir_type()
    ) {
        let typedef = HirTypedef::new(name, base_type.clone());
        prop_assert_eq!(typedef.underlying_type(), &base_type);
    }

    /// Property 4: Pointer typedefs should have pointer underlying type
    #[test]
    fn prop_hir_pointer_typedef_is_pointer(
        name in valid_identifier(),
        inner_type in simple_hir_type()
    ) {
        let pointer_type = HirType::Pointer(Box::new(inner_type.clone()));
        let typedef = HirTypedef::new(name, pointer_type.clone());

        prop_assert_eq!(typedef.underlying_type(), &pointer_type);

        // Verify it's a pointer type
        match typedef.underlying_type() {
            HirType::Pointer(inner) => {
                prop_assert_eq!(inner.as_ref(), &inner_type);
            }
            _ => prop_assert!(false, "Expected pointer type"),
        }
    }

    /// Property 5: Multiple typedefs with same name should be independent
    #[test]
    fn prop_hir_typedefs_independent(
        name in valid_identifier(),
    ) {
        let typedef1 = HirTypedef::new(name.clone(), HirType::Int);
        let typedef2 = HirTypedef::new(name.clone(), HirType::Float);

        // Same names
        prop_assert_eq!(typedef1.name(), typedef2.name());

        // Different types
        prop_assert_ne!(typedef1.underlying_type(), typedef2.underlying_type());
    }

    /// Property 6: HirTypedef creation should be deterministic
    #[test]
    fn prop_hir_typedef_deterministic(
        name in valid_identifier(),
        base_type in simple_hir_type()
    ) {
        let typedef1 = HirTypedef::new(name.clone(), base_type.clone());
        let typedef2 = HirTypedef::new(name.clone(), base_type.clone());

        prop_assert_eq!(typedef1.name(), typedef2.name());
        prop_assert_eq!(typedef1.underlying_type(), typedef2.underlying_type());
    }

    /// Property 7: Struct typedefs should preserve struct name
    #[test]
    fn prop_hir_struct_typedef_preserves_name(
        typedef_name in valid_identifier(),
        struct_name in valid_identifier(),
    ) {
        let typedef = HirTypedef::new(
            typedef_name.clone(),
            HirType::Struct(struct_name.clone())
        );

        prop_assert_eq!(typedef.name(), typedef_name.as_str());

        match typedef.underlying_type() {
            HirType::Struct(name) => {
                prop_assert_eq!(name, &struct_name);
            }
            _ => prop_assert!(false, "Expected struct type"),
        }
    }

    /// Property 8: Function pointer typedefs should preserve param types
    #[test]
    fn prop_hir_function_pointer_typedef_preserves_params(
        typedef_name in valid_identifier(),
        param_count in 0usize..5,
    ) {
        let param_types = vec![HirType::Int; param_count];
        let return_type = Box::new(HirType::Int);

        let typedef = HirTypedef::new(
            typedef_name.clone(),
            HirType::FunctionPointer {
                param_types: param_types.clone(),
                return_type: return_type.clone(),
            }
        );

        match typedef.underlying_type() {
            HirType::FunctionPointer { param_types: params, return_type: ret } => {
                prop_assert_eq!(params.len(), param_count);
                prop_assert_eq!(ret, &return_type);
            }
            _ => prop_assert!(false, "Expected function pointer type"),
        }
    }

    /// Property 9: Typedef names can be any valid identifier
    #[test]
    fn prop_hir_typedef_accepts_any_identifier(
        name in valid_identifier(),
    ) {
        let typedef = HirTypedef::new(name.clone(), HirType::Int);
        prop_assert_eq!(typedef.name(), name.as_str());

        // Name should not be empty
        prop_assert!(!name.is_empty());

        // Name should start with letter or underscore
        let first_char = name.chars().next().unwrap();
        prop_assert!(first_char.is_alphabetic() || first_char == '_');
    }

    /// Property 10: Nested pointer typedefs should work
    #[test]
    fn prop_hir_nested_pointer_typedef(
        name in valid_identifier(),
        base_type in simple_hir_type()
    ) {
        // Create pointer to pointer
        let inner_pointer = HirType::Pointer(Box::new(base_type.clone()));
        let outer_pointer = HirType::Pointer(Box::new(inner_pointer.clone()));

        let typedef = HirTypedef::new(name, outer_pointer.clone());

        prop_assert_eq!(typedef.underlying_type(), &outer_pointer);

        // Verify structure: Pointer(Pointer(base_type))
        match typedef.underlying_type() {
            HirType::Pointer(outer) => {
                match outer.as_ref() {
                    HirType::Pointer(inner) => {
                        prop_assert_eq!(inner.as_ref(), &base_type);
                    }
                    _ => prop_assert!(false, "Expected nested pointer"),
                }
            }
            _ => prop_assert!(false, "Expected outer pointer"),
        }
    }
}
