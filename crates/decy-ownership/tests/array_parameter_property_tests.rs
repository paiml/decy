//! Property-based tests for array parameter detection
//!
//! DECY-073 RED: These tests verify the robustness of array parameter detection
//! across edge cases and random inputs.
//!
//! Target: 20 properties × 1000 cases = 20,000 test cases

use decy_hir::{HirFunction, HirParameter, HirStatement, HirType, HirExpression};
use decy_ownership::dataflow::DataflowAnalyzer;
use proptest::prelude::*;

// ============================================================================
// PROPERTY 1: Array parameters with length always detected
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_pointer_with_int_length_is_array(
        arr_suffix in "[a-z]{0,5}",
        len_suffix in "[a-z]{0,5}",
    ) {
        // Use common array name to boost confidence
        let arr_name = format!("arr{}", arr_suffix);
        let len_name = format!("len{}", len_suffix);

        // Ensure names are different
        prop_assume!(arr_name != len_name);

        // Generate: fn foo(arr: *T, len: int)
        let params = vec![
            HirParameter::new(arr_name.to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new(len_name, HirType::Int),
        ];

        let func = HirFunction::new("test".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Should detect as array parameter (common name + int param = 2 signals)
        assert_eq!(
            graph.is_array_parameter(&arr_name),
            Some(true),
            "Pointer with common name followed by int should be detected as array"
        );
    }
}

// ============================================================================
// PROPERTY 2: Single pointers without length NOT detected as array
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_single_pointer_not_array(ptr_name in "[a-z]{3,8}") {
        // Generate: fn foo(ptr: *T)
        let params = vec![
            HirParameter::new(ptr_name.to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ];

        let func = HirFunction::new("test".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Should NOT detect as array parameter
        assert_eq!(
            graph.is_array_parameter(&ptr_name),
            Some(false),
            "Single pointer without length should NOT be array"
        );
    }
}

// ============================================================================
// PROPERTY 3: Common array names boost confidence
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_common_array_names_detected(
        suffix in "[a-z]{0,5}",
        prefix in prop::sample::select(vec!["arr", "array", "buf", "buffer", "data", "items"]),
    ) {
        let arr_name = format!("{}{}", prefix, suffix);
        let params = vec![
            HirParameter::new(arr_name.to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ];

        let func = HirFunction::new("test".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Should detect (name pattern + length param)
        assert_eq!(
            graph.is_array_parameter(&arr_name),
            Some(true),
            "Common array name '{}' with length should be detected", arr_name
        );
    }
}

// ============================================================================
// PROPERTY 4: Common length parameter names boost confidence
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_common_length_names_detected(
        arr_name in "[a-z]{3,8}",
        len_name in prop::sample::select(vec!["len", "length", "size", "count", "num"]),
    ) {
        let params = vec![
            HirParameter::new(arr_name.to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new(len_name.to_string(), HirType::Int),
        ];

        let func = HirFunction::new("test".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Should detect (length name pattern + int param)
        assert_eq!(
            graph.is_array_parameter(&arr_name),
            Some(true),
            "Pointer with common length name should be detected"
        );
    }
}

// ============================================================================
// PROPERTY 5: Array indexing in body increases confidence
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_array_indexing_detected(
        arr_name in "[a-z]{3,8}",
        index_var in "[a-z]{1,3}",
    ) {
        let params = vec![
            HirParameter::new(arr_name.to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ];

        // Create body with array indexing: arr[i] = 0
        let body = vec![
            HirStatement::ArrayIndexAssignment {
                array: Box::new(HirExpression::Variable(arr_name.clone())),
                index: Box::new(HirExpression::Variable(index_var)),
                value: HirExpression::IntLiteral(0),
            },
        ];

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            params,
            body,
        );

        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Should strongly detect (length param + array indexing)
        assert_eq!(
            graph.is_array_parameter(&arr_name),
            Some(true),
            "Pointer with array indexing should be detected"
        );
    }
}

// ============================================================================
// PROPERTY 6: Multiple array parameters all detected
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_multiple_arrays_detected(
        arr1 in "[a-z]{3,6}",
        arr2 in "[a-z]{3,6}",
    ) {
        // Ensure names are different
        prop_assume!(arr1 != arr2);

        let params = vec![
            HirParameter::new(arr1.clone(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len1".to_string(), HirType::Int),
            HirParameter::new(arr2.clone(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len2".to_string(), HirType::Int),
        ];

        let func = HirFunction::new("test".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Both should be detected
        assert_eq!(
            graph.is_array_parameter(&arr1),
            Some(true),
            "First array parameter should be detected"
        );
        assert_eq!(
            graph.is_array_parameter(&arr2),
            Some(true),
            "Second array parameter should be detected"
        );
    }
}

// ============================================================================
// PROPERTY 7: Char pointers with size are arrays
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_char_pointer_with_size_is_array(
        buf_suffix in "[a-z]{0,5}",
        size_suffix in "[a-z]{0,5}",
    ) {
        // Use common buffer name to boost confidence
        let buf_name = format!("buf{}", buf_suffix);
        let size_name = format!("size{}", size_suffix);

        // Ensure names are different
        prop_assume!(buf_name != size_name);

        let params = vec![
            HirParameter::new(buf_name.to_string(), HirType::Pointer(Box::new(HirType::Char))),
            HirParameter::new(size_name.to_string(), HirType::Int),
        ];

        let func = HirFunction::new("test".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Char pointer with size should be detected as buffer (common name + int param + size name = 3 signals)
        assert_eq!(
            graph.is_array_parameter(&buf_name),
            Some(true),
            "Char pointer with common names should be detected as array"
        );
    }
}

// ============================================================================
// PROPERTY 8: Struct pointers NOT detected as arrays
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_struct_pointer_not_array(
        ptr_name in "[a-z]{3,8}",
        struct_name in "[A-Z][a-z]{3,8}",
    ) {
        let params = vec![
            HirParameter::new(
                ptr_name.clone(),
                HirType::Pointer(Box::new(HirType::Struct(struct_name)))
            ),
            HirParameter::new("len".to_string(), HirType::Int),
        ];

        let func = HirFunction::new("test".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Struct pointers should NOT be detected as arrays (ambiguous)
        assert_eq!(
            graph.is_array_parameter(&ptr_name),
            Some(false),
            "Struct pointer should NOT be detected as array"
        );
    }
}

// ============================================================================
// PROPERTY 9: Non-pointer parameters never detected as arrays
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_non_pointer_not_array(
        param_name in "[a-z]{3,8}",
        param_type in prop::sample::select(vec![HirType::Int, HirType::Float, HirType::Char]),
    ) {
        let params = vec![
            HirParameter::new(param_name.to_string(), param_type),
            HirParameter::new("len".to_string(), HirType::Int),
        ];

        let func = HirFunction::new("test".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Non-pointer should never be array
        assert_eq!(
            graph.is_array_parameter(&param_name),
            Some(false),
            "Non-pointer parameter should NOT be detected as array"
        );
    }
}

// ============================================================================
// PROPERTY 10: Pointer followed by non-int less likely to be array
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_pointer_with_non_int_weak_array_signal(
        arr_name in "[a-z]{3,8}",
        second_param in "[a-z]{3,8}",
    ) {
        prop_assume!(arr_name != second_param);

        let params = vec![
            HirParameter::new(arr_name.to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new(second_param.to_string(), HirType::Float),  // Not an int!
        ];

        let func = HirFunction::new("test".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Without int length parameter, detection is less certain
        // This tests the heuristic behavior
        let result = graph.is_array_parameter(&arr_name);
        assert!(result.is_some(), "Should return Some (not None)");
    }
}

// ============================================================================
// PROPERTY 11: Parameter order matters - length must follow pointer
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_reversed_order_different_detection(
        arr_name in "[a-z]{3,8}",
        len_name in "[a-z]{3,8}",
    ) {
        prop_assume!(arr_name != len_name);

        // Reversed: len first, then pointer
        let params = vec![
            HirParameter::new(len_name.clone(), HirType::Int),
            HirParameter::new(arr_name.to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ];

        let func = HirFunction::new("test".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // The integer parameter should NOT be detected as array
        assert_eq!(
            graph.is_array_parameter(&len_name),
            Some(false),
            "Int parameter should NOT be array"
        );
    }
}

// ============================================================================
// PROPERTY 12: Empty function body still allows detection
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_empty_body_detection_works(
        arr_name in "[a-z]{3,8}",
    ) {
        let params = vec![
            HirParameter::new(arr_name.to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ];

        // Empty body
        let func = HirFunction::new("test".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Should still detect based on signature alone
        assert_eq!(
            graph.is_array_parameter(&arr_name),
            Some(true),
            "Detection should work with empty body"
        );
    }
}

// ============================================================================
// PROPERTY 13: Different pointer types all work
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_different_element_types_detected(
        arr_name in "[a-z]{3,8}",
        element_type in prop::sample::select(vec![
            HirType::Int,
            HirType::Float,
            HirType::Double,
            HirType::Char,
        ]),
    ) {
        let params = vec![
            HirParameter::new(arr_name.to_string(), HirType::Pointer(Box::new(element_type))),
            HirParameter::new("len".to_string(), HirType::Int),
        ];

        let func = HirFunction::new("test".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // All pointer types with length should be detected
        assert_eq!(
            graph.is_array_parameter(&arr_name),
            Some(true),
            "Pointer of any type with length should be array"
        );
    }
}

// ============================================================================
// PROPERTY 14: Case sensitivity in names handled correctly
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_case_insensitive_name_matching(
        prefix in prop::sample::select(vec!["Arr", "ARR", "arr", "Array", "ARRAY"]),
        suffix in "[a-z]{0,4}",
    ) {
        let arr_name = format!("{}{}", prefix, suffix);
        let params = vec![
            HirParameter::new(arr_name.to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ];

        let func = HirFunction::new("test".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Case variations of "arr" should all be detected
        assert_eq!(
            graph.is_array_parameter(&arr_name),
            Some(true),
            "Case variations of array names should be detected"
        );
    }
}

// ============================================================================
// PROPERTY 15: Function with only array params all detected
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_all_array_params_detected(
        count in 1usize..=4,
    ) {
        // Generate multiple array parameters
        let mut params = Vec::new();
        let mut arr_names = Vec::new();

        for i in 0..count {
            let arr_name = format!("arr{}", i);
            let len_name = format!("len{}", i);
            arr_names.push(arr_name.clone());
            params.push(HirParameter::new(arr_name, HirType::Pointer(Box::new(HirType::Int))));
            params.push(HirParameter::new(len_name.to_string(), HirType::Int));
        }

        let func = HirFunction::new("test".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // All array parameters should be detected
        for arr_name in &arr_names {
            assert_eq!(
                graph.is_array_parameter(arr_name),
                Some(true),
                "Array parameter '{}' should be detected", arr_name
            );
        }
    }
}

// ============================================================================
// PROPERTY 16: Detection is consistent across multiple calls
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_detection_idempotent(
        arr_name in "[a-z]{3,8}",
    ) {
        let params = vec![
            HirParameter::new(arr_name.to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ];

        let func = HirFunction::new("test".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Call detection multiple times
        let result1 = graph.is_array_parameter(&arr_name);
        let result2 = graph.is_array_parameter(&arr_name);
        let result3 = graph.is_array_parameter(&arr_name);

        // Results should be identical
        assert_eq!(result1, result2, "Detection should be consistent");
        assert_eq!(result2, result3, "Detection should be consistent");
    }
}

// ============================================================================
// PROPERTY 17: Unknown parameter names return None
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_unknown_param_returns_none(
        arr_name in "[a-z]{3,8}",
        unknown_name in "[a-z]{3,8}",
    ) {
        prop_assume!(arr_name != unknown_name);

        let params = vec![
            HirParameter::new(arr_name, HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ];

        let func = HirFunction::new("test".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Unknown parameter should return None
        assert_eq!(
            graph.is_array_parameter(&unknown_name),
            None,
            "Unknown parameter should return None"
        );
    }
}

// ============================================================================
// PROPERTY 18: Array detection with complex body
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_complex_body_detection(
        arr_name in "[a-z]{3,8}",
    ) {
        let params = vec![
            HirParameter::new(arr_name.to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ];

        // Complex body with nested if statements
        let body = vec![
            HirStatement::If {
                condition: HirExpression::IntLiteral(1),
                then_block: vec![
                    HirStatement::ArrayIndexAssignment {
                        array: Box::new(HirExpression::Variable(arr_name.clone())),
                        index: Box::new(HirExpression::IntLiteral(0)),
                        value: HirExpression::IntLiteral(42),
                    },
                ],
                else_block: None,
            },
        ];

        let func = HirFunction::new_with_body(
            "test".to_string(),
            HirType::Void,
            params,
            body,
        );

        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Should detect even with complex body
        assert_eq!(
            graph.is_array_parameter(&arr_name),
            Some(true),
            "Detection should work with complex body"
        );
    }
}

// ============================================================================
// PROPERTY 19: Last parameter can be array
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_last_param_can_be_array(
        first_param in "[a-z]{3,8}",
        arr_name in "[a-z]{3,8}",
    ) {
        prop_assume!(first_param != arr_name);

        let params = vec![
            HirParameter::new(first_param.to_string(), HirType::Int),
            HirParameter::new(arr_name.to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ];

        let func = HirFunction::new("test".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Array parameter not in first position should still be detected
        assert_eq!(
            graph.is_array_parameter(&arr_name),
            Some(true),
            "Array parameter in any position should be detected"
        );
    }
}

// ============================================================================
// PROPERTY 20: Pointer without following int has weak signals
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_pointer_as_last_param_weak_signal(
        arr_name in "[a-z]{3,8}",
    ) {
        // Pointer as last parameter (no following length)
        let params = vec![
            HirParameter::new("first".to_string(), HirType::Int),
            HirParameter::new(arr_name.to_string(), HirType::Pointer(Box::new(HirType::Int))),
        ];

        let func = HirFunction::new("test".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Without length parameter following, should not be detected as array
        assert_eq!(
            graph.is_array_parameter(&arr_name),
            Some(false),
            "Pointer without following length should NOT be detected as array"
        );
    }
}
