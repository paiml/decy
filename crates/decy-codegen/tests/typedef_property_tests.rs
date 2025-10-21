//! Property tests for typedef code generation (DECY-023 REFACTOR phase)
//!
//! This test suite uses proptest to verify typedef → type alias generation.
//! Target: 10 properties × 256 cases = 2,560 test cases.
//!
//! References:
//! - K&R §6.7: Type Names
//! - ISO C99 §6.7.7: Type definitions

use decy_codegen::CodeGenerator;
use decy_hir::{HirType, HirTypedef};
use proptest::prelude::*;

/// Generate valid Rust identifier strings
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
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Property 1: Code generation should never panic for any typedef
    #[test]
    fn prop_codegen_typedef_never_panics(
        name in valid_identifier(),
        base_type in simple_hir_type()
    ) {
        let typedef = HirTypedef::new(name, base_type);
        let generator = CodeGenerator::new();

        // Should not panic
        let _ = generator.generate_typedef(&typedef);
    }

    /// Property 2: Generated code should always contain the typedef name
    #[test]
    fn prop_codegen_contains_typedef_name(
        name in valid_identifier(),
        base_type in simple_hir_type()
    ) {
        let typedef = HirTypedef::new(name.clone(), base_type);
        let generator = CodeGenerator::new();

        if let Ok(code) = generator.generate_typedef(&typedef) {
            prop_assert!(
                code.contains(&name),
                "Generated code should contain typedef name '{}', got: {}",
                name,
                code
            );
        }
    }

    /// Property 3: Generated code should have "type" keyword
    #[test]
    fn prop_codegen_contains_type_keyword(
        name in valid_identifier(),
        base_type in simple_hir_type()
    ) {
        let typedef = HirTypedef::new(name, base_type);
        let generator = CodeGenerator::new();

        if let Ok(code) = generator.generate_typedef(&typedef) {
            prop_assert!(
                code.contains("type "),
                "Generated code should contain 'type ' keyword, got: {}",
                code
            );
        }
    }

    /// Property 4: Generated code should have "pub" visibility
    #[test]
    fn prop_codegen_typedef_is_public(
        name in valid_identifier(),
        base_type in simple_hir_type()
    ) {
        let typedef = HirTypedef::new(name, base_type);
        let generator = CodeGenerator::new();

        if let Ok(code) = generator.generate_typedef(&typedef) {
            // Should have pub unless it's a redundant typedef (commented out)
            prop_assert!(
                code.contains("pub type ") || code.contains("// type "),
                "Generated code should be public or commented, got: {}",
                code
            );
        }
    }

    /// Property 5: Int typedef should generate i32
    #[test]
    fn prop_codegen_int_typedef_generates_i32(
        name in valid_identifier(),
    ) {
        let typedef = HirTypedef::new(name, HirType::Int);
        let generator = CodeGenerator::new();

        if let Ok(code) = generator.generate_typedef(&typedef) {
            prop_assert!(
                code.contains("i32"),
                "Int typedef should generate i32, got: {}",
                code
            );
        }
    }

    /// Property 6: Float typedef should generate f32
    #[test]
    fn prop_codegen_float_typedef_generates_f32(
        name in valid_identifier(),
    ) {
        let typedef = HirTypedef::new(name, HirType::Float);
        let generator = CodeGenerator::new();

        if let Ok(code) = generator.generate_typedef(&typedef) {
            prop_assert!(
                code.contains("f32"),
                "Float typedef should generate f32, got: {}",
                code
            );
        }
    }

    /// Property 7: Code generation should be deterministic
    #[test]
    fn prop_codegen_typedef_deterministic(
        name in valid_identifier(),
        base_type in simple_hir_type()
    ) {
        let typedef1 = HirTypedef::new(name.clone(), base_type.clone());
        let typedef2 = HirTypedef::new(name, base_type);

        let generator1 = CodeGenerator::new();
        let generator2 = CodeGenerator::new();

        let code1 = generator1.generate_typedef(&typedef1);
        let code2 = generator2.generate_typedef(&typedef2);

        prop_assert_eq!(code1.is_ok(), code2.is_ok());

        if let (Ok(c1), Ok(c2)) = (code1, code2) {
            prop_assert_eq!(c1, c2, "Code generation should be deterministic");
        }
    }

    /// Property 8: Pointer typedef should contain pointer syntax
    #[test]
    fn prop_codegen_pointer_typedef_has_pointer(
        name in valid_identifier(),
        base_type in simple_hir_type()
    ) {
        let typedef = HirTypedef::new(
            name,
            HirType::Pointer(Box::new(base_type))
        );
        let generator = CodeGenerator::new();

        if let Ok(code) = generator.generate_typedef(&typedef) {
            prop_assert!(
                code.contains("*mut "),
                "Pointer typedef should contain '*mut ', got: {}",
                code
            );
        }
    }

    /// Property 9: Struct typedef should preserve struct name
    #[test]
    fn prop_codegen_struct_typedef_preserves_name(
        typedef_name in valid_identifier(),
        struct_name in valid_identifier(),
    ) {
        // Only test if names are different (otherwise it's redundant)
        prop_assume!(typedef_name != struct_name);

        let typedef = HirTypedef::new(
            typedef_name.clone(),
            HirType::Struct(struct_name.clone())
        );
        let generator = CodeGenerator::new();

        if let Ok(code) = generator.generate_typedef(&typedef) {
            prop_assert!(
                code.contains(&typedef_name),
                "Should contain typedef name '{}', got: {}",
                typedef_name,
                code
            );
            prop_assert!(
                code.contains(&struct_name),
                "Should contain struct name '{}', got: {}",
                struct_name,
                code
            );
        }
    }

    /// Property 10: Function pointer typedef should contain "fn"
    #[test]
    fn prop_codegen_function_pointer_typedef_contains_fn(
        name in valid_identifier(),
        param_count in 0usize..5,
    ) {
        let param_types = vec![HirType::Int; param_count];
        let typedef = HirTypedef::new(
            name,
            HirType::FunctionPointer {
                param_types,
                return_type: Box::new(HirType::Int),
            }
        );
        let generator = CodeGenerator::new();

        if let Ok(code) = generator.generate_typedef(&typedef) {
            prop_assert!(
                code.contains("fn("),
                "Function pointer typedef should contain 'fn(', got: {}",
                code
            );
        }
    }
}
