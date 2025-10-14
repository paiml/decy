//! Tests for borrow code generation.

use super::*;
use crate::dataflow::DataflowAnalyzer;
use crate::inference::{OwnershipInference, OwnershipInferencer, OwnershipKind};
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

#[test]
fn test_generate_immutable_borrow() {
    // DECY-041: Test that pointers are now preserved as raw pointers for pointer arithmetic
    // Old behavior: ImmutableBorrow inference generated &T type
    // New behavior: All pointers stay as *mut T to support pointer arithmetic
    let generator = BorrowGenerator::new();

    let ptr_type = HirType::Pointer(Box::new(HirType::Int));
    let mut inferences = HashMap::new();
    inferences.insert(
        "data".to_string(),
        OwnershipInference {
            variable: "data".to_string(),
            kind: OwnershipKind::ImmutableBorrow,
            confidence: 0.8,
            reason: "Test inference".to_string(),
        },
    );

    let transformed = generator.transform_type(&ptr_type, "data", &inferences);

    assert_eq!(
        transformed,
        HirType::Pointer(Box::new(HirType::Int)),
        "DECY-041: Pointers are preserved as *mut T for pointer arithmetic support"
    );
}

#[test]
fn test_generate_mutable_borrow() {
    // DECY-041: Test that pointers are now preserved as raw pointers for pointer arithmetic
    // Old behavior: MutableBorrow inference generated &mut T type
    // New behavior: All pointers stay as *mut T to support pointer arithmetic
    let generator = BorrowGenerator::new();

    let ptr_type = HirType::Pointer(Box::new(HirType::Int));
    let mut inferences = HashMap::new();
    inferences.insert(
        "data".to_string(),
        OwnershipInference {
            variable: "data".to_string(),
            kind: OwnershipKind::MutableBorrow,
            confidence: 0.8,
            reason: "Test inference".to_string(),
        },
    );

    let transformed = generator.transform_type(&ptr_type, "data", &inferences);

    assert_eq!(
        transformed,
        HirType::Pointer(Box::new(HirType::Int)),
        "DECY-041: Pointers are preserved as *mut T for pointer arithmetic support"
    );
}

#[test]
fn test_generate_borrowed_parameter() {
    // DECY-041: Test that pointer parameters are preserved as raw pointers
    // Old behavior: Parameters transformed to references based on inference
    // New behavior: All pointer parameters stay as *mut T to support pointer arithmetic
    let generator = BorrowGenerator::new();

    let params = vec![HirParameter::new(
        "data".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
    )];

    let mut inferences = HashMap::new();
    inferences.insert(
        "data".to_string(),
        OwnershipInference {
            variable: "data".to_string(),
            kind: OwnershipKind::ImmutableBorrow,
            confidence: 0.8,
            reason: "Parameter is read-only".to_string(),
        },
    );

    let transformed_params = generator.transform_parameters(&params, &inferences);

    assert_eq!(transformed_params.len(), 1);
    assert_eq!(
        transformed_params[0].param_type(),
        &HirType::Pointer(Box::new(HirType::Int)),
        "DECY-041: Pointer parameters are preserved as *mut T for pointer arithmetic support"
    );
}

#[test]
fn test_borrow_checker_validation() {
    // DECY-041: Test that pointers are preserved as raw pointers
    // Old behavior: Generated references to follow borrow checker rules
    // New behavior: All pointers stay as *mut T to support pointer arithmetic
    let generator = BorrowGenerator::new();

    let ptr_type = HirType::Pointer(Box::new(HirType::Int));
    let mut inferences = HashMap::new();

    // Multiple immutable borrows inference (but preserved as raw pointers)
    inferences.insert(
        "data1".to_string(),
        OwnershipInference {
            variable: "data1".to_string(),
            kind: OwnershipKind::ImmutableBorrow,
            confidence: 0.8,
            reason: "Immutable borrow 1".to_string(),
        },
    );
    inferences.insert(
        "data2".to_string(),
        OwnershipInference {
            variable: "data2".to_string(),
            kind: OwnershipKind::ImmutableBorrow,
            confidence: 0.8,
            reason: "Immutable borrow 2".to_string(),
        },
    );

    let transformed1 = generator.transform_type(&ptr_type, "data1", &inferences);
    let transformed2 = generator.transform_type(&ptr_type, "data2", &inferences);

    // DECY-041: Both should be raw pointers for pointer arithmetic support
    assert!(matches!(transformed1, HirType::Pointer(..)));
    assert!(matches!(transformed2, HirType::Pointer(..)));
}

#[test]
fn test_end_to_end_borrow_generation() {
    // DECY-041: End-to-end test with pointer arithmetic support
    // Old behavior: analyze function, infer ownership, generate references
    // New behavior: pointers are preserved as raw pointers for pointer arithmetic
    let func = HirFunction::new_with_body(
        "read_only".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::Dereference(
            Box::new(HirExpression::Variable("data".to_string())),
        )))],
    );

    // Analyze dataflow
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    // Infer ownership
    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    // Generate borrow code
    let generator = BorrowGenerator::new();
    let transformed_func = generator.transform_function(&func, &inferences);

    // Verify transformation
    assert_eq!(transformed_func.parameters().len(), 1);
    assert!(
        matches!(
            transformed_func.parameters()[0].param_type(),
            HirType::Pointer(..)
        ),
        "DECY-041: Pointer parameters are preserved as *mut T for pointer arithmetic support"
    );
}

#[test]
fn test_owning_pointer_becomes_box() {
    // DECY-041: Test that pointers are preserved as raw pointers
    // Old behavior: Owning pointers transformed to Box<T>
    // New behavior: All pointers stay as *mut T to support pointer arithmetic
    // Future: Could optimize owning pointers to Box<T> when no pointer arithmetic detected
    let generator = BorrowGenerator::new();

    let ptr_type = HirType::Pointer(Box::new(HirType::Int));
    let mut inferences = HashMap::new();
    inferences.insert(
        "ptr".to_string(),
        OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Owning,
            confidence: 0.9,
            reason: "malloc allocation".to_string(),
        },
    );

    let transformed = generator.transform_type(&ptr_type, "ptr", &inferences);

    assert_eq!(
        transformed,
        HirType::Pointer(Box::new(HirType::Int)),
        "DECY-041: All pointers preserved as *mut T for pointer arithmetic support"
    );
}

#[test]
fn test_unknown_ownership_stays_raw_pointer() {
    // Test that Unknown ownership keeps raw pointer type
    let generator = BorrowGenerator::new();

    let ptr_type = HirType::Pointer(Box::new(HirType::Int));
    let mut inferences = HashMap::new();
    inferences.insert(
        "ptr".to_string(),
        OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Unknown,
            confidence: 0.3,
            reason: "Uncertain".to_string(),
        },
    );

    let transformed = generator.transform_type(&ptr_type, "ptr", &inferences);

    assert_eq!(
        transformed,
        HirType::Pointer(Box::new(HirType::Int)),
        "Unknown ownership should keep raw pointer"
    );
}

#[test]
fn test_non_pointer_type_unchanged() {
    // Test that non-pointer types are not affected by borrow generation
    let generator = BorrowGenerator::new();

    let int_type = HirType::Int;
    let inferences = HashMap::new();

    let transformed = generator.transform_type(&int_type, "x", &inferences);

    assert_eq!(
        transformed,
        HirType::Int,
        "Non-pointer types should remain unchanged"
    );
}

// RED PHASE: Enhanced borrow generation tests

#[test]
fn test_multiple_immutable_borrows_allowed() {
    // DECY-041: Test that pointer parameters are preserved as raw pointers
    // Old behavior: Multiple immutable borrows generated &T references
    // New behavior: All pointers stay as *mut T to support pointer arithmetic
    let generator = BorrowGenerator::new();

    let params = vec![
        HirParameter::new("data".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        HirParameter::new(
            "data2".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        ),
    ];

    let mut inferences = HashMap::new();
    inferences.insert(
        "data".to_string(),
        OwnershipInference {
            variable: "data".to_string(),
            kind: OwnershipKind::ImmutableBorrow,
            confidence: 0.8,
            reason: "First immutable borrow".to_string(),
        },
    );
    inferences.insert(
        "data2".to_string(),
        OwnershipInference {
            variable: "data2".to_string(),
            kind: OwnershipKind::ImmutableBorrow,
            confidence: 0.8,
            reason: "Second immutable borrow".to_string(),
        },
    );

    let transformed = generator.transform_parameters(&params, &inferences);

    // DECY-041: Both should be raw pointers for pointer arithmetic support
    assert_eq!(transformed.len(), 2);
    assert!(matches!(transformed[0].param_type(), HirType::Pointer(..)));
    assert!(matches!(transformed[1].param_type(), HirType::Pointer(..)));
}

#[test]
fn test_mutable_borrow_prevents_other_borrows() {
    // DECY-041: Test that pointer parameters are preserved as raw pointers
    // Old behavior: Generated &mut T and &T references (conflict detection future enhancement)
    // New behavior: All pointers stay as *mut T to support pointer arithmetic
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "modify_data".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("data".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new(
                "data2".to_string(),
                HirType::Pointer(Box::new(HirType::Int)),
            ),
        ],
        vec![],
    );

    let mut inferences = HashMap::new();
    inferences.insert(
        "data".to_string(),
        OwnershipInference {
            variable: "data".to_string(),
            kind: OwnershipKind::MutableBorrow,
            confidence: 0.9,
            reason: "Mutable borrow".to_string(),
        },
    );
    inferences.insert(
        "data2".to_string(),
        OwnershipInference {
            variable: "data2".to_string(),
            kind: OwnershipKind::ImmutableBorrow,
            confidence: 0.8,
            reason: "Immutable borrow of same data".to_string(),
        },
    );

    let transformed = generator.transform_function(&func, &inferences);

    // DECY-041: Both should be raw pointers for pointer arithmetic support
    assert_eq!(transformed.parameters().len(), 2);
    assert!(matches!(
        transformed.parameters()[0].param_type(),
        HirType::Pointer(..)
    ));
    assert!(matches!(
        transformed.parameters()[1].param_type(),
        HirType::Pointer(..)
    ));
}

#[test]
fn test_nested_pointer_types() {
    // DECY-041: Test that nested pointers are preserved as raw pointers
    // Old behavior: int** → &&T (nested references)
    // New behavior: All pointers stay as *mut T to support pointer arithmetic
    let generator = BorrowGenerator::new();

    let nested_ptr_type = HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Int))));
    let mut inferences = HashMap::new();
    inferences.insert(
        "data".to_string(),
        OwnershipInference {
            variable: "data".to_string(),
            kind: OwnershipKind::ImmutableBorrow,
            confidence: 0.7,
            reason: "Nested pointer borrow".to_string(),
        },
    );

    let transformed = generator.transform_type(&nested_ptr_type, "data", &inferences);

    // DECY-041: Nested pointers stay as raw pointers for pointer arithmetic support
    assert!(matches!(transformed, HirType::Pointer(..)));
}

#[test]
fn test_lifetime_aware_borrow_generation() {
    // DECY-041: Test that pointers are preserved for pointer arithmetic
    // Old behavior: Lifetime-aware reference generation
    // New behavior: All pointers stay as *mut T to support pointer arithmetic
    // Future phase: add lifetime tracking for when references can be safely used
    let func = HirFunction::new_with_body(
        "get_value".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new(
            "container".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::Variable(
            "container".to_string(),
        )))],
    );

    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);

    let inferencer = OwnershipInferencer::new();
    let inferences = inferencer.infer(&graph);

    let generator = BorrowGenerator::new();
    let transformed = generator.transform_function(&func, &inferences);

    // DECY-041: Parameter should be preserved as raw pointer
    assert!(matches!(
        transformed.parameters()[0].param_type(),
        HirType::Pointer(..)
    ));

    // Return type should also be preserved as pointer
    assert!(matches!(transformed.return_type(), HirType::Pointer(..)));
}

#[test]
fn test_high_confidence_borrows_prioritized() {
    // DECY-041: Test that pointers are preserved regardless of confidence
    // Old behavior: High-confidence inferences generated references
    // New behavior: All pointers stay as *mut T to support pointer arithmetic
    let generator = BorrowGenerator::new();

    let ptr_type = HirType::Pointer(Box::new(HirType::Int));
    let mut inferences = HashMap::new();
    inferences.insert(
        "data".to_string(),
        OwnershipInference {
            variable: "data".to_string(),
            kind: OwnershipKind::ImmutableBorrow,
            confidence: 0.95, // Very high confidence
            reason: "Const qualifier and read-only usage".to_string(),
        },
    );

    let transformed = generator.transform_type(&ptr_type, "data", &inferences);

    // DECY-041: Even high confidence stays as raw pointer for pointer arithmetic
    assert_eq!(
        transformed,
        HirType::Pointer(Box::new(HirType::Int)),
        "DECY-041: All pointers preserved as *mut T for pointer arithmetic support"
    );
}

#[test]
fn test_low_confidence_falls_back_to_raw_pointer() {
    // Test that low-confidence Unknown inferences keep raw pointers
    // This is safer than guessing - better to use unsafe than incorrectly infer
    let generator = BorrowGenerator::new();

    let ptr_type = HirType::Pointer(Box::new(HirType::Int));
    let mut inferences = HashMap::new();
    inferences.insert(
        "data".to_string(),
        OwnershipInference {
            variable: "data".to_string(),
            kind: OwnershipKind::Unknown,
            confidence: 0.25, // Very low confidence
            reason: "Uncertain usage pattern".to_string(),
        },
    );

    let transformed = generator.transform_type(&ptr_type, "data", &inferences);

    // Low confidence Unknown should fall back to raw pointer
    assert_eq!(transformed, HirType::Pointer(Box::new(HirType::Int)));
}

// REFACTOR PHASE: Property tests for borrow generation

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_immutable_borrow_generates_non_mutable_reference(
            var_name in "[a-z][a-z0-9_]{0,10}",
        ) {
            // DECY-041: Property: ImmutableBorrow inference preserves raw pointers
            // Old behavior: Always generated non-mutable reference
            // New behavior: All pointers stay as *mut T to support pointer arithmetic
            let generator = BorrowGenerator::new();
            let ptr_type = HirType::Pointer(Box::new(HirType::Int));
            let mut inferences = HashMap::new();
            inferences.insert(
                var_name.clone(),
                OwnershipInference {
                    variable: var_name.clone(),
                    kind: OwnershipKind::ImmutableBorrow,
                    confidence: 0.8,
                    reason: "Test".to_string(),
                },
            );

            let transformed = generator.transform_type(&ptr_type, &var_name, &inferences);

            // DECY-041: Should preserve as raw pointer for pointer arithmetic
            let is_raw_ptr = matches!(transformed, HirType::Pointer(..));
            prop_assert!(is_raw_ptr);
        }

        #[test]
        fn prop_mutable_borrow_generates_mutable_reference(
            var_name in "[a-z][a-z0-9_]{0,10}",
        ) {
            // DECY-041: Property: MutableBorrow inference preserves raw pointers
            // Old behavior: Always generated mutable reference
            // New behavior: All pointers stay as *mut T to support pointer arithmetic
            let generator = BorrowGenerator::new();
            let ptr_type = HirType::Pointer(Box::new(HirType::Int));
            let mut inferences = HashMap::new();
            inferences.insert(
                var_name.clone(),
                OwnershipInference {
                    variable: var_name.clone(),
                    kind: OwnershipKind::MutableBorrow,
                    confidence: 0.8,
                    reason: "Test".to_string(),
                },
            );

            let transformed = generator.transform_type(&ptr_type, &var_name, &inferences);

            // DECY-041: Should preserve as raw pointer for pointer arithmetic
            let is_raw_ptr = matches!(transformed, HirType::Pointer(..));
            prop_assert!(is_raw_ptr);
        }

        #[test]
        fn prop_owning_generates_box(
            var_name in "[a-z][a-z0-9_]{0,10}",
        ) {
            // DECY-041: Property: Owning inference preserves raw pointers
            // Old behavior: Owning pointers always became Box<T>
            // New behavior: All pointers stay as *mut T to support pointer arithmetic
            let generator = BorrowGenerator::new();
            let ptr_type = HirType::Pointer(Box::new(HirType::Int));
            let mut inferences = HashMap::new();
            inferences.insert(
                var_name.clone(),
                OwnershipInference {
                    variable: var_name.clone(),
                    kind: OwnershipKind::Owning,
                    confidence: 0.9,
                    reason: "Test".to_string(),
                },
            );

            let transformed = generator.transform_type(&ptr_type, &var_name, &inferences);

            // DECY-041: Should preserve as raw pointer for pointer arithmetic
            prop_assert!(matches!(transformed, HirType::Pointer(..)));
        }

        #[test]
        fn prop_unknown_keeps_raw_pointer(
            var_name in "[a-z][a-z0-9_]{0,10}",
        ) {
            // Property: Unknown ownership preserves raw pointer
            let generator = BorrowGenerator::new();
            let ptr_type = HirType::Pointer(Box::new(HirType::Int));
            let mut inferences = HashMap::new();
            inferences.insert(
                var_name.clone(),
                OwnershipInference {
                    variable: var_name.clone(),
                    kind: OwnershipKind::Unknown,
                    confidence: 0.3,
                    reason: "Test".to_string(),
                },
            );

            let transformed = generator.transform_type(&ptr_type, &var_name, &inferences);

            prop_assert!(matches!(transformed, HirType::Pointer(..)));
        }

        #[test]
        fn prop_transformation_deterministic(
            var_name in "[a-z][a-z0-9_]{0,10}",
            confidence in 0.0f32..1.0f32,
        ) {
            // DECY-041: Property: Same input produces same output (now raw pointers)
            // Old behavior: Deterministic reference generation
            // New behavior: Deterministic raw pointer preservation
            let generator = BorrowGenerator::new();
            let ptr_type = HirType::Pointer(Box::new(HirType::Int));
            let mut inferences = HashMap::new();
            inferences.insert(
                var_name.clone(),
                OwnershipInference {
                    variable: var_name.clone(),
                    kind: OwnershipKind::ImmutableBorrow,
                    confidence,
                    reason: "Test".to_string(),
                },
            );

            let transformed1 = generator.transform_type(&ptr_type, &var_name, &inferences);
            let transformed2 = generator.transform_type(&ptr_type, &var_name, &inferences);

            prop_assert_eq!(transformed1, transformed2);
        }

        #[test]
        fn prop_non_pointer_types_unchanged(
            var_name in "[a-z][a-z0-9_]{0,10}",
        ) {
            // Property: Non-pointer types are never transformed
            let generator = BorrowGenerator::new();
            let non_ptr_types = vec![HirType::Int, HirType::Float, HirType::Void];
            let inferences = HashMap::new();

            for ty in non_ptr_types {
                let transformed = generator.transform_type(&ty, &var_name, &inferences);
                prop_assert_eq!(&transformed, &ty);
            }
        }

        #[test]
        fn prop_transform_never_panics(
            var_name in "[a-z][a-z0-9_]{0,10}",
            confidence in 0.0f32..1.0f32,
        ) {
            // Property: Transformation never panics
            let generator = BorrowGenerator::new();
            let ptr_type = HirType::Pointer(Box::new(HirType::Int));

            // Try all ownership kinds
            for kind in &[
                OwnershipKind::Owning,
                OwnershipKind::ImmutableBorrow,
                OwnershipKind::MutableBorrow,
                OwnershipKind::Unknown,
            ] {
                let mut inferences = HashMap::new();
                inferences.insert(
                    var_name.clone(),
                    OwnershipInference {
                        variable: var_name.clone(),
                        kind: kind.clone(),
                        confidence,
                        reason: "Test".to_string(),
                    },
                );

                // Should not panic
                let _transformed = generator.transform_type(&ptr_type, &var_name, &inferences);
            }
        }

        #[test]
        fn prop_parameter_transformation_preserves_count(
            num_params in 0usize..5,
            param_names in prop::collection::vec("[a-z][a-z0-9_]{0,10}", 0..5),
        ) {
            // Property: Transformation preserves parameter count
            let generator = BorrowGenerator::new();

            let params: Vec<HirParameter> = param_names
                .iter()
                .take(num_params)
                .map(|name| {
                    HirParameter::new(
                        name.clone(),
                        HirType::Pointer(Box::new(HirType::Int)),
                    )
                })
                .collect();

            let inferences = HashMap::new();
            let transformed = generator.transform_parameters(&params, &inferences);

            prop_assert_eq!(transformed.len(), params.len());
        }
    }
}
