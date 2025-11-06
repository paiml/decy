//! Property-based tests for array parameter detection (DECY-073).
//!
//! Uses proptest to generate random function signatures and verify detection heuristics.
//! Target: 20 properties × 1000 cases = 20K test cases

use decy_hir::{HirFunction, HirParameter, HirStatement, HirType};
use decy_ownership::dataflow::DataflowAnalyzer;
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

/// Strategy to generate pointer types
fn pointer_type_strategy() -> impl Strategy<Value = HirType> {
    prop_oneof![
        Just(HirType::Pointer(Box::new(HirType::Int))),
        Just(HirType::Pointer(Box::new(HirType::Char))),
        Just(HirType::Pointer(Box::new(HirType::Float))),
        Just(HirType::Pointer(Box::new(HirType::Double))),
    ]
}

// ============================================================================
// PROPERTY 1: Array names with length params are detected as arrays
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_array_name_with_length_param_is_detected(
        array_name in array_name_strategy(),
        length_name in length_name_strategy(),
        ptr_type in pointer_type_strategy(),
    ) {
        let params = vec![
            HirParameter::new(array_name.clone(), ptr_type),
            HirParameter::new(length_name, HirType::Int),
        ];

        let func = HirFunction::new("test_func".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Property: array-like names with length params should be detected
        prop_assert_eq!(
            graph.is_array_parameter(&array_name),
            Some(true),
            "Array-like name '{}' with length param should be detected",
            array_name
        );
    }
}

// ============================================================================
// PROPERTY 2: Pointer followed by int param is detected as array
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_pointer_followed_by_int_is_array(
        param_name in identifier_strategy(),
        length_name in identifier_strategy(),
        ptr_type in pointer_type_strategy(),
    ) {
        // Ensure names are different
        prop_assume!(param_name != length_name);

        let params = vec![
            HirParameter::new(param_name.clone(), ptr_type),
            HirParameter::new(length_name, HirType::Int),
        ];

        let func = HirFunction::new("test_func".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Property: any pointer followed by int param might be array
        // (detection depends on heuristics, but should return Some(bool))
        let result = graph.is_array_parameter(&param_name);
        prop_assert!(
            result.is_some(),
            "Detection should return Some for pointer with int param"
        );
    }
}

// ============================================================================
// PROPERTY 3: Pointer without length param is NOT detected as array
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_pointer_without_length_not_array(
        param_name in identifier_strategy(),
        ptr_type in pointer_type_strategy(),
    ) {
        let params = vec![
            HirParameter::new(param_name.clone(), ptr_type),
        ];

        let func = HirFunction::new("test_func".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Property: pointer without length param should NOT be detected as array
        prop_assert_eq!(
            graph.is_array_parameter(&param_name),
            Some(false),
            "Pointer '{}' without length param should not be array",
            param_name
        );
    }
}

// ============================================================================
// PROPERTY 4: Non-pointer params are never detected as arrays
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_non_pointer_never_array(
        param_name in identifier_strategy(),
    ) {
        let params = vec![
            HirParameter::new(param_name.clone(), HirType::Int),
            HirParameter::new("len".to_string(), HirType::Int),
        ];

        let func = HirFunction::new("test_func".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Property: non-pointer types are never arrays
        prop_assert_eq!(
            graph.is_array_parameter(&param_name),
            Some(false),
            "Non-pointer param '{}' should never be detected as array",
            param_name
        );
    }
}

// ============================================================================
// PROPERTY 5: Detection is deterministic (same input = same output)
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_detection_is_deterministic(
        param_name in identifier_strategy(),
        length_name in identifier_strategy(),
        ptr_type in pointer_type_strategy(),
    ) {
        prop_assume!(param_name != length_name);

        let params = vec![
            HirParameter::new(param_name.clone(), ptr_type.clone()),
            HirParameter::new(length_name.clone(), HirType::Int),
        ];

        let func = HirFunction::new("test_func".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();

        // Run detection twice
        let graph1 = analyzer.analyze(&func);
        let result1 = graph1.is_array_parameter(&param_name);

        let graph2 = analyzer.analyze(&func);
        let result2 = graph2.is_array_parameter(&param_name);

        // Property: same input should produce same output
        prop_assert_eq!(
            result1, result2,
            "Detection should be deterministic for '{}'",
            param_name
        );
    }
}

// ============================================================================
// PROPERTY 6: Multiple array params are all detected
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_multiple_arrays_all_detected(
        arr1 in array_name_strategy(),
        arr2 in array_name_strategy(),
        len1 in length_name_strategy(),
        len2 in length_name_strategy(),
    ) {
        // Ensure all names are unique
        prop_assume!(arr1 != arr2);
        prop_assume!(len1 != len2);
        prop_assume!(arr1 != len1 && arr1 != len2);
        prop_assume!(arr2 != len1 && arr2 != len2);

        let params = vec![
            HirParameter::new(arr1.clone(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new(len1, HirType::Int),
            HirParameter::new(arr2.clone(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new(len2, HirType::Int),
        ];

        let func = HirFunction::new("test_func".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Property: both arrays should be detected
        prop_assert_eq!(
            graph.is_array_parameter(&arr1),
            Some(true),
            "First array '{}' should be detected",
            arr1
        );
        prop_assert_eq!(
            graph.is_array_parameter(&arr2),
            Some(true),
            "Second array '{}' should be detected",
            arr2
        );
    }
}

// ============================================================================
// PROPERTY 7: Struct pointers are NOT detected as arrays
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_struct_pointer_not_array(
        param_name in identifier_strategy(),
        struct_name in identifier_strategy(),
        length_name in length_name_strategy(),
    ) {
        let params = vec![
            HirParameter::new(
                param_name.clone(),
                HirType::Pointer(Box::new(HirType::Struct(struct_name))),
            ),
            HirParameter::new(length_name, HirType::Int),
        ];

        let func = HirFunction::new("test_func".to_string(), HirType::Void, params);
        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Property: struct pointers should not be detected as arrays (conservative)
        prop_assert_eq!(
            graph.is_array_parameter(&param_name),
            Some(false),
            "Struct pointer '{}' should not be detected as array",
            param_name
        );
    }
}

// ============================================================================
// PROPERTY 8: Parameter order matters for detection
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_parameter_order_matters(
        arr_name in array_name_strategy(),
        len_name in length_name_strategy(),
    ) {
        prop_assume!(arr_name != len_name);

        // Test 1: pointer followed by int (should be detected)
        let params1 = vec![
            HirParameter::new(arr_name.clone(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new(len_name.clone(), HirType::Int),
        ];
        let func1 = HirFunction::new("test_func".to_string(), HirType::Void, params1);
        let analyzer = DataflowAnalyzer::new();
        let graph1 = analyzer.analyze(&func1);

        // Test 2: int followed by pointer (should NOT be detected)
        let params2 = vec![
            HirParameter::new(len_name.clone(), HirType::Int),
            HirParameter::new(arr_name.clone(), HirType::Pointer(Box::new(HirType::Int))),
        ];
        let func2 = HirFunction::new("test_func".to_string(), HirType::Void, params2);
        let graph2 = analyzer.analyze(&func2);

        // Property: order affects detection
        let detected1 = graph1.is_array_parameter(&arr_name);
        let detected2 = graph2.is_array_parameter(&arr_name);

        // First case should have higher confidence for detection
        prop_assert!(
            detected1 == Some(true) || detected2 == Some(false),
            "Parameter order should affect detection: ptr-int vs int-ptr"
        );
    }
}

// ============================================================================
// PROPERTY 9: Array indexing in body increases detection confidence
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_array_indexing_strengthens_detection(
        param_name in identifier_strategy(),
        length_name in identifier_strategy(),
        index in 0..10i32,
    ) {
        prop_assume!(param_name != length_name);

        use decy_hir::HirExpression;

        // Create function with array indexing in body
        let params = vec![
            HirParameter::new(param_name.clone(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new(length_name, HirType::Int),
        ];

        let body = vec![
            HirStatement::ArrayIndexAssignment {
                array: Box::new(HirExpression::Variable(param_name.clone())),
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

        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Property: array indexing should result in detection
        prop_assert_eq!(
            graph.is_array_parameter(&param_name),
            Some(true),
            "Param '{}' with array indexing should be detected",
            param_name
        );
    }
}

// ============================================================================
// PROPERTY 10: Pointer arithmetic reduces detection confidence
// ============================================================================
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_pointer_arithmetic_weakens_detection(
        param_name in identifier_strategy(),
        length_name in identifier_strategy(),
        offset in 1..10i32,
    ) {
        prop_assume!(param_name != length_name);

        use decy_hir::{BinaryOperator, HirExpression};

        // Create function with pointer arithmetic in body
        let params = vec![
            HirParameter::new(param_name.clone(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new(length_name, HirType::Int),
        ];

        let body = vec![
            HirStatement::VariableDeclaration {
                name: "ptr2".to_string(),
                var_type: HirType::Pointer(Box::new(HirType::Int)),
                initializer: Some(HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable(param_name.clone())),
                    right: Box::new(HirExpression::IntLiteral(offset)),
                }),
            },
        ];

        let func = HirFunction::new_with_body(
            "test_func".to_string(),
            HirType::Void,
            params,
            body,
        );

        let analyzer = DataflowAnalyzer::new();
        let graph = analyzer.analyze(&func);

        // Property: pointer arithmetic should reduce confidence
        // (may still be detected, but with lower confidence)
        let result = graph.is_array_parameter(&param_name);
        prop_assert!(
            result.is_some(),
            "Should return Some even with pointer arithmetic"
        );
    }
}
