//! Property-based tests for typedef code generation (DECY-023 REFACTOR phase).

use proptest::prelude::*;

use super::*;
use decy_hir::{HirType, HirTypedef};

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
    fn test_generated_typedef_contains_name(name in typedef_name_strategy(), hir_type in hir_type_strategy()) {
        let codegen = CodeGenerator::new();
        let typedef = HirTypedef::new(name.clone(), hir_type);
        let code = codegen.generate_typedef(&typedef);

        prop_assert!(code.contains(&name), "Generated code should contain typedef name: {}", code);
    }

    #[test]
    fn test_generated_typedef_is_not_empty(name in typedef_name_strategy(), hir_type in hir_type_strategy()) {
        let codegen = CodeGenerator::new();
        let typedef = HirTypedef::new(name, hir_type);
        let code = codegen.generate_typedef(&typedef);

        prop_assert!(!code.is_empty(), "Generated code should not be empty");
    }

    #[test]
    fn test_simple_typedef_has_type_keyword(name in typedef_name_strategy(), hir_type in hir_type_strategy()) {
        let codegen = CodeGenerator::new();
        let typedef = HirTypedef::new(name, hir_type);
        let code = codegen.generate_typedef(&typedef);

        // Either it's a regular type alias or a comment
        prop_assert!(
            code.starts_with("type ") || code.starts_with("//"),
            "Generated code should start with 'type' or '//' (comment): {}",
            code
        );
    }

    #[test]
    fn test_simple_typedef_ends_with_semicolon_or_paren(name in typedef_name_strategy(), hir_type in hir_type_strategy()) {
        let codegen = CodeGenerator::new();
        let typedef = HirTypedef::new(name, hir_type);
        let code = codegen.generate_typedef(&typedef);

        // Either ends with ; (regular typedef) or ) (comment)
        prop_assert!(
            code.ends_with(';') || code.ends_with(')'),
            "Generated code should end with ';' or ')': {}",
            code
        );
    }

    #[test]
    fn test_int_typedef_contains_i32(name in typedef_name_strategy()) {
        let codegen = CodeGenerator::new();
        let typedef = HirTypedef::new(name, HirType::Int);
        let code = codegen.generate_typedef(&typedef);

        prop_assert!(code.contains("i32"), "Int typedef should contain 'i32': {}", code);
    }

    #[test]
    fn test_float_typedef_contains_f32(name in typedef_name_strategy()) {
        let codegen = CodeGenerator::new();
        let typedef = HirTypedef::new(name, HirType::Float);
        let code = codegen.generate_typedef(&typedef);

        prop_assert!(code.contains("f32"), "Float typedef should contain 'f32': {}", code);
    }

    #[test]
    fn test_double_typedef_contains_f64(name in typedef_name_strategy()) {
        let codegen = CodeGenerator::new();
        let typedef = HirTypedef::new(name, HirType::Double);
        let code = codegen.generate_typedef(&typedef);

        prop_assert!(code.contains("f64"), "Double typedef should contain 'f64': {}", code);
    }

    #[test]
    fn test_redundant_struct_typedef_is_comment(name in typedef_name_strategy()) {
        let codegen = CodeGenerator::new();
        // Create typedef where name == underlying struct name
        let typedef = HirTypedef::new(name.clone(), HirType::Struct(name));
        let code = codegen.generate_typedef(&typedef);

        prop_assert!(code.starts_with("//"), "Redundant struct typedef should be a comment: {}", code);
        prop_assert!(code.contains("redundant"), "Comment should mention 'redundant': {}", code);
    }
}
