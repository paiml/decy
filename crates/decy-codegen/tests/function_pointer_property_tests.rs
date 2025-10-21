//! Property tests for function pointer code generation (DECY-024 TDD-Refactor phase)
//!
//! This test suite uses proptest to verify function pointer → fn type generation.
//! Target: 10 properties × 256 cases = 2,560 test cases.
//!
//! References:
//! - K&R §5.11: Pointers to Functions
//! - ISO C99 §6.7.5.3: Function declarators

use decy_codegen::CodeGenerator;
use decy_hir::HirType;
use proptest::prelude::*;

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

    /// Property 1: Code generation should never panic
    #[test]
    fn prop_codegen_never_panics(
        param_type in simple_hir_type(),
        return_type in simple_hir_type(),
    ) {
        let fn_ptr = HirType::FunctionPointer {
            param_types: vec![param_type],
            return_type: Box::new(return_type),
        };

        let _ = CodeGenerator::map_type(&fn_ptr);
    }

    /// Property 2: Generated code should always contain "fn"
    #[test]
    fn prop_codegen_contains_fn(
        param_type in simple_hir_type(),
        return_type in simple_hir_type(),
    ) {
        let fn_ptr = HirType::FunctionPointer {
            param_types: vec![param_type],
            return_type: Box::new(return_type),
        };

        let rust_code = CodeGenerator::map_type(&fn_ptr);

        prop_assert!(
            rust_code.contains("fn"),
            "Generated code should contain 'fn', got: {}",
            rust_code
        );
    }

    /// Property 3: Generated code should contain arrow for non-void return
    #[test]
    fn prop_codegen_has_arrow_for_non_void(
        param_type in simple_hir_type(),
        return_type in simple_hir_type(),
    ) {
        let fn_ptr = HirType::FunctionPointer {
            param_types: vec![param_type],
            return_type: Box::new(return_type),
        };

        let rust_code = CodeGenerator::map_type(&fn_ptr);

        prop_assert!(
            rust_code.contains("->"),
            "Non-void return should have '->', got: {}",
            rust_code
        );
    }

    /// Property 4: Void return should not have arrow
    #[test]
    fn prop_codegen_no_arrow_for_void(
        param_type in simple_hir_type(),
    ) {
        let fn_ptr = HirType::FunctionPointer {
            param_types: vec![param_type],
            return_type: Box::new(HirType::Void),
        };

        let rust_code = CodeGenerator::map_type(&fn_ptr);

        prop_assert!(
            !rust_code.contains("->"),
            "Void return should not have '->', got: {}",
            rust_code
        );
    }

    /// Property 5: Code generation should be deterministic
    #[test]
    fn prop_codegen_deterministic(
        param_type in simple_hir_type(),
        return_type in simple_hir_type(),
    ) {
        let fn_ptr = HirType::FunctionPointer {
            param_types: vec![param_type],
            return_type: Box::new(return_type),
        };

        let code1 = CodeGenerator::map_type(&fn_ptr);
        let code2 = CodeGenerator::map_type(&fn_ptr);

        prop_assert_eq!(code1, code2, "Code generation should be deterministic");
    }

    /// Property 6: Parameter count should affect generated code
    #[test]
    fn prop_codegen_param_count_affects_output(
        param_count in 0usize..5,
    ) {
        let param_types = vec![HirType::Int; param_count];

        let fn_ptr = HirType::FunctionPointer {
            param_types,
            return_type: Box::new(HirType::Int),
        };

        let rust_code = CodeGenerator::map_type(&fn_ptr);

        // Should have correct number of commas (param_count - 1)
        let comma_count = rust_code.matches(',').count();
        if param_count > 0 {
            prop_assert_eq!(comma_count, param_count - 1);
        }
    }

    /// Property 7: Int param should generate i32
    #[test]
    fn prop_codegen_int_param_generates_i32(
        return_type in simple_hir_type(),
    ) {
        let fn_ptr = HirType::FunctionPointer {
            param_types: vec![HirType::Int],
            return_type: Box::new(return_type),
        };

        let rust_code = CodeGenerator::map_type(&fn_ptr);

        prop_assert!(
            rust_code.contains("i32"),
            "Int param should generate i32, got: {}",
            rust_code
        );
    }

    /// Property 8: Float param should generate f32
    #[test]
    fn prop_codegen_float_param_generates_f32(
        return_type in simple_hir_type(),
    ) {
        let fn_ptr = HirType::FunctionPointer {
            param_types: vec![HirType::Float],
            return_type: Box::new(return_type),
        };

        let rust_code = CodeGenerator::map_type(&fn_ptr);

        prop_assert!(
            rust_code.contains("f32"),
            "Float param should generate f32, got: {}",
            rust_code
        );
    }

    /// Property 9: Generated code should have matching parentheses
    #[test]
    fn prop_codegen_balanced_parens(
        param_type in simple_hir_type(),
        return_type in simple_hir_type(),
    ) {
        let fn_ptr = HirType::FunctionPointer {
            param_types: vec![param_type],
            return_type: Box::new(return_type),
        };

        let rust_code = CodeGenerator::map_type(&fn_ptr);

        let open_count = rust_code.matches('(').count();
        let close_count = rust_code.matches(')').count();

        prop_assert_eq!(open_count, close_count, "Parentheses should be balanced");
    }

    /// Property 10: Empty param list should generate fn()
    #[test]
    fn prop_codegen_empty_params_generates_fn_parens(
        return_type in simple_hir_type(),
    ) {
        let fn_ptr = HirType::FunctionPointer {
            param_types: vec![],
            return_type: Box::new(return_type),
        };

        let rust_code = CodeGenerator::map_type(&fn_ptr);

        prop_assert!(
            rust_code.contains("fn()"),
            "Empty params should generate fn(), got: {}",
            rust_code
        );
    }
}
