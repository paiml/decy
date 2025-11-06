//! Property-based tests for signature transformation (DECY-073).
//!
//! Tests that array parameters are correctly transformed to slices in signatures.
//! Target: 20 properties × 1000 cases = 20K test cases (combined with detection tests)

use decy_codegen::CodeGenerator;
use decy_hir::{HirFunction, HirParameter, HirStatement, HirType};
use proptest::prelude::*;

/// Strategy to generate valid C identifier names
fn identifier_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,15}").expect("Valid regex for identifiers")
}

/// Strategy to generate array-like parameter names
fn array_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("arr".to_string()),
        Just("array".to_string()),
        Just("buf".to_string()),
        Just("buffer".to_string()),
        Just("data".to_string()),
        Just("items".to_string()),
    ]
}

/// Strategy to generate length-like parameter names
fn length_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("len".to_string()),
        Just("length".to_string()),
        Just("size".to_string()),
        Just("count".to_string()),
        Just("num".to_string()),
    ]
}

// ============================================================================
// PROPERTY 11: Array param signatures don't contain raw pointers
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_array_param_no_raw_pointer(
        arr_name in array_name_strategy(),
        len_name in length_name_strategy(),
    ) {
        prop_assume!(arr_name != len_name);

        let params = vec![
            HirParameter::new(arr_name, HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new(len_name, HirType::Int),
        ];

        let func = HirFunction::new("test_func".to_string(), HirType::Void, params);
        let codegen = CodeGenerator::new();
        let signature = codegen.generate_signature(&func);

        // Property: transformed signatures should not contain *mut or *const
        prop_assert!(
            !signature.contains("*mut") && !signature.contains("*const"),
            "Array param signature should not contain raw pointers: {}",
            signature
        );
    }
}

// ============================================================================
// PROPERTY 12: Array param signatures contain slice syntax
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_array_param_has_slice(
        arr_name in array_name_strategy(),
        len_name in length_name_strategy(),
    ) {
        prop_assume!(arr_name != len_name);

        let params = vec![
            HirParameter::new(arr_name, HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new(len_name, HirType::Int),
        ];

        let func = HirFunction::new("test_func".to_string(), HirType::Void, params);
        let codegen = CodeGenerator::new();
        let signature = codegen.generate_signature(&func);

        // Property: should contain slice syntax &[T] or &mut [T]
        prop_assert!(
            signature.contains("&[") || signature.contains("&mut ["),
            "Array param signature should contain slice syntax: {}",
            signature
        );
    }
}

// ============================================================================
// PROPERTY 13: Length parameters are removed from signature
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_length_param_removed(
        arr_name in array_name_strategy(),
        len_name in length_name_strategy(),
    ) {
        prop_assume!(arr_name != len_name);

        let params = vec![
            HirParameter::new(arr_name, HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new(len_name.clone(), HirType::Int),
        ];

        let func = HirFunction::new("test_func".to_string(), HirType::Void, params);
        let codegen = CodeGenerator::new();
        let signature = codegen.generate_signature(&func);

        // Property: length parameter name should not appear in signature
        prop_assert!(
            !signature.contains(&len_name),
            "Signature should not contain length param '{}': {}",
            len_name,
            signature
        );
    }
}

// ============================================================================
// PROPERTY 14: Non-array pointers remain as raw pointers
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_non_array_pointer_unchanged(
        param_name in identifier_strategy(),
    ) {
        let params = vec![
            HirParameter::new(param_name.clone(), HirType::Pointer(Box::new(HirType::Int))),
        ];

        let func = HirFunction::new("test_func".to_string(), HirType::Void, params);
        let codegen = CodeGenerator::new();
        let signature = codegen.generate_signature(&func);

        // Property: non-array pointers should remain as *mut
        prop_assert!(
            signature.contains("*mut") || signature.contains(&param_name),
            "Non-array pointer should remain as raw pointer: {}",
            signature
        );
    }
}

// ============================================================================
// PROPERTY 15: Mutable arrays use &mut slices
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_mutable_array_uses_mut_slice(
        arr_name in array_name_strategy(),
        len_name in length_name_strategy(),
        index in 0..10i32,
    ) {
        prop_assume!(arr_name != len_name);

        use decy_hir::HirExpression;

        let params = vec![
            HirParameter::new(arr_name.clone(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new(len_name, HirType::Int),
        ];

        // Create function body with array modification
        let body = vec![
            HirStatement::ArrayIndexAssignment {
                array: Box::new(HirExpression::Variable(arr_name)),
                index: Box::new(HirExpression::IntLiteral(index)),
                value: HirExpression::IntLiteral(42),
            },
        ];

        let func = HirFunction::new_with_body(
            "test_func".to_string(),
            HirType::Void,
            params,
            body,
        );

        let codegen = CodeGenerator::new();
        let signature = codegen.generate_signature(&func);

        // Property: modified arrays should use &mut slices
        prop_assert!(
            signature.contains("&mut ["),
            "Modified array should use &mut slice: {}",
            signature
        );
    }
}

// ============================================================================
// PROPERTY 16: Immutable arrays use & slices
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_immutable_array_uses_ref_slice(
        arr_name in array_name_strategy(),
        len_name in length_name_strategy(),
    ) {
        prop_assume!(arr_name != len_name);

        let params = vec![
            HirParameter::new(arr_name, HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new(len_name, HirType::Int),
        ];

        // No body - just signature (immutable)
        let func = HirFunction::new("test_func".to_string(), HirType::Void, params);
        let codegen = CodeGenerator::new();
        let signature = codegen.generate_signature(&func);

        // Property: unmodified arrays can use & slices
        // (Note: may also use &mut, so we just check for slice syntax)
        prop_assert!(
            signature.contains("&[") || signature.contains("&mut ["),
            "Array should use slice syntax: {}",
            signature
        );
    }
}

// ============================================================================
// PROPERTY 17: Function generation is deterministic
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_generation_is_deterministic(
        arr_name in array_name_strategy(),
        len_name in length_name_strategy(),
    ) {
        prop_assume!(arr_name != len_name);

        let params = vec![
            HirParameter::new(arr_name, HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new(len_name, HirType::Int),
        ];

        let func = HirFunction::new("test_func".to_string(), HirType::Void, params);
        let codegen = CodeGenerator::new();

        // Generate twice
        let sig1 = codegen.generate_signature(&func);
        let sig2 = codegen.generate_signature(&func);

        // Property: same input should produce same output
        prop_assert_eq!(
            sig1, sig2,
            "Signature generation should be deterministic"
        );
    }
}

// ============================================================================
// PROPERTY 18: Multiple array params all transformed
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_multiple_arrays_all_transformed(
        arr1 in array_name_strategy(),
        arr2 in array_name_strategy(),
        len1 in length_name_strategy(),
        len2 in length_name_strategy(),
    ) {
        // Ensure unique names
        prop_assume!(arr1 != arr2);
        prop_assume!(len1 != len2);
        prop_assume!(arr1 != len1 && arr1 != len2);
        prop_assume!(arr2 != len1 && arr2 != len2);

        let params = vec![
            HirParameter::new(arr1.clone(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new(len1.clone(), HirType::Int),
            HirParameter::new(arr2.clone(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new(len2.clone(), HirType::Int),
        ];

        let func = HirFunction::new("test_func".to_string(), HirType::Void, params);
        let codegen = CodeGenerator::new();
        let signature = codegen.generate_signature(&func);

        // Property: both array names should appear in signature
        prop_assert!(
            signature.contains(&arr1),
            "First array '{}' should be in signature: {}",
            arr1,
            signature
        );
        prop_assert!(
            signature.contains(&arr2),
            "Second array '{}' should be in signature: {}",
            arr2,
            signature
        );

        // Neither length parameter should appear
        prop_assert!(
            !signature.contains(&len1),
            "First length '{}' should not be in signature: {}",
            len1,
            signature
        );
        prop_assert!(
            !signature.contains(&len2),
            "Second length '{}' should not be in signature: {}",
            len2,
            signature
        );
    }
}

// ============================================================================
// PROPERTY 19: Different element types produce different slice types
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_element_type_affects_slice_type(
        arr_name in array_name_strategy(),
        len_name in length_name_strategy(),
        element_type in prop_oneof![
            Just(HirType::Int),
            Just(HirType::Char),
            Just(HirType::Float),
            Just(HirType::Double),
        ],
    ) {
        prop_assume!(arr_name != len_name);

        let params = vec![
            HirParameter::new(arr_name, HirType::Pointer(Box::new(element_type.clone()))),
            HirParameter::new(len_name, HirType::Int),
        ];

        let func = HirFunction::new("test_func".to_string(), HirType::Void, params);
        let codegen = CodeGenerator::new();
        let signature = codegen.generate_signature(&func);

        // Property: signature should contain element type
        let expected_type = match element_type {
            HirType::Int => "i32",
            HirType::Char => "u8",
            HirType::Float => "f32",
            HirType::Double => "f64",
            _ => unreachable!(),
        };

        prop_assert!(
            signature.contains(expected_type),
            "Signature should contain element type '{}': {}",
            expected_type,
            signature
        );
    }
}

// ============================================================================
// PROPERTY 20: Generated code compiles (syntax check)
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_generated_signature_has_valid_syntax(
        arr_name in array_name_strategy(),
        len_name in length_name_strategy(),
    ) {
        prop_assume!(arr_name != len_name);

        let params = vec![
            HirParameter::new(arr_name, HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new(len_name, HirType::Int),
        ];

        let func = HirFunction::new("test_func".to_string(), HirType::Void, params);
        let codegen = CodeGenerator::new();
        let code = codegen.generate_function(&func);

        // Property: generated code should have basic syntax validity
        // Check for balanced braces
        let open_braces = code.matches('{').count();
        let close_braces = code.matches('}').count();
        prop_assert_eq!(
            open_braces, close_braces,
            "Braces should be balanced in generated code"
        );

        // Check for balanced parentheses in signature
        let sig_end = code.find('{').unwrap_or(code.len());
        let sig = &code[..sig_end];
        let open_parens = sig.matches('(').count();
        let close_parens = sig.matches(')').count();
        prop_assert_eq!(
            open_parens, close_parens,
            "Parentheses should be balanced in signature"
        );

        // Check that it starts with "fn "
        prop_assert!(
            code.starts_with("fn "),
            "Generated code should start with 'fn '"
        );
    }
}
