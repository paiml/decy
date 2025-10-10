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
}
