//! Property-based tests for typedef support (DECY-023 REFACTOR phase).

use proptest::prelude::*;

use super::*;

// Strategy for generating valid typedef names (alphanumeric, starting with letter)
fn typedef_name_strategy() -> impl Strategy<Value = String> {
    "[A-Z][a-zA-Z0-9_]{0,20}".prop_map(|s| s.to_string())
}

// Strategy for generating HirType (simplified for property testing)
fn hir_type_strategy() -> impl Strategy<Value = HirType> {
    prop_oneof![
        Just(HirType::Void),
        Just(HirType::Int),
        Just(HirType::Float),
        Just(HirType::Double),
        Just(HirType::Char),
    ]
}

proptest! {
    #[test]
    fn test_typedef_name_roundtrip(name in typedef_name_strategy(), hir_type in hir_type_strategy()) {
        let typedef = HirTypedef::new(name.clone(), hir_type.clone());
        prop_assert_eq!(typedef.name(), &name);
        prop_assert_eq!(typedef.underlying_type(), &hir_type);
    }

    #[test]
    fn test_typedef_clone_equals_original(name in typedef_name_strategy(), hir_type in hir_type_strategy()) {
        let typedef = HirTypedef::new(name, hir_type);
        let cloned = typedef.clone();
        prop_assert_eq!(typedef, cloned);
    }

    #[test]
    fn test_typedef_equality_is_reflexive(name in typedef_name_strategy(), hir_type in hir_type_strategy()) {
        let typedef = HirTypedef::new(name, hir_type);
        prop_assert_eq!(&typedef, &typedef);
    }

    #[test]
    fn test_typedef_equality_is_symmetric(name in typedef_name_strategy(), hir_type in hir_type_strategy()) {
        let typedef1 = HirTypedef::new(name.clone(), hir_type.clone());
        let typedef2 = HirTypedef::new(name, hir_type);
        prop_assert_eq!(&typedef1, &typedef2);
        prop_assert_eq!(&typedef2, &typedef1);
    }

    #[test]
    fn test_typedef_different_names_not_equal(
        name1 in typedef_name_strategy(),
        name2 in typedef_name_strategy(),
        hir_type in hir_type_strategy()
    ) {
        prop_assume!(name1 != name2);
        let typedef1 = HirTypedef::new(name1, hir_type.clone());
        let typedef2 = HirTypedef::new(name2, hir_type);
        prop_assert_ne!(&typedef1, &typedef2);
    }
}
