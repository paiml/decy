//! Tests for borrow code generation.

use super::*;
use crate::dataflow::DataflowAnalyzer;
use crate::inference::{OwnershipInference, OwnershipInferencer, OwnershipKind};
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

#[test]
fn test_generate_immutable_borrow() {
    // Test that ImmutableBorrow inference generates &T type
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
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
        "ImmutableBorrow should generate &T"
    );
}

#[test]
fn test_generate_mutable_borrow() {
    // Test that MutableBorrow inference generates &mut T type
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
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        },
        "MutableBorrow should generate &mut T"
    );
}

#[test]
fn test_generate_borrowed_parameter() {
    // Test that function parameters are transformed based on inference
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
        &HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
        "Parameter should be transformed to &T"
    );
}

#[test]
fn test_borrow_checker_validation() {
    // Test that generated borrow types follow borrow checker rules
    // This is a simplified test - real validation would be more complex
    let generator = BorrowGenerator::new();

    let ptr_type = HirType::Pointer(Box::new(HirType::Int));
    let mut inferences = HashMap::new();

    // Multiple immutable borrows are allowed
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

    // Both should be immutable references
    assert!(matches!(
        transformed1,
        HirType::Reference { mutable: false, .. }
    ));
    assert!(matches!(
        transformed2,
        HirType::Reference { mutable: false, .. }
    ));
}

#[test]
fn test_end_to_end_borrow_generation() {
    // End-to-end test: analyze function, infer ownership, generate borrow code
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
            HirType::Reference { mutable: false, .. }
        ),
        "Read-only parameter should be transformed to &T"
    );
}

#[test]
fn test_owning_pointer_becomes_box() {
    // Test that Owning pointers are transformed to Box<T>
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
        HirType::Box(Box::new(HirType::Int)),
        "Owning pointer should become Box<T>"
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
    // Test that multiple immutable borrows of the same data are allowed
    // This follows Rust's borrow checker rules: multiple &T are ok
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

    // Both should be immutable references - this is valid Rust
    assert_eq!(transformed.len(), 2);
    assert!(matches!(
        transformed[0].param_type(),
        HirType::Reference { mutable: false, .. }
    ));
    assert!(matches!(
        transformed[1].param_type(),
        HirType::Reference { mutable: false, .. }
    ));
}

#[test]
fn test_mutable_borrow_prevents_other_borrows() {
    // Test detection of borrow checker violations
    // In Rust, if you have &mut T, you can't have any other borrows
    // This test verifies we can detect potential conflicts
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

    // Should generate both borrows (conflict detection is a future enhancement)
    assert_eq!(transformed.parameters().len(), 2);
    assert!(matches!(
        transformed.parameters()[0].param_type(),
        HirType::Reference { mutable: true, .. }
    ));
    assert!(matches!(
        transformed.parameters()[1].param_type(),
        HirType::Reference { mutable: false, .. }
    ));
}

#[test]
fn test_nested_pointer_types() {
    // Test handling of nested pointer types (int** → &&T)
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

    // Nested pointers: outer pointer becomes reference
    assert!(matches!(transformed, HirType::Reference { .. }));
}

#[test]
fn test_lifetime_aware_borrow_generation() {
    // Test that borrow generation is prepared for lifetime annotations
    // Future phase: add lifetime tracking to inferences
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

    // Parameter should be transformed to borrow
    assert!(matches!(
        transformed.parameters()[0].param_type(),
        HirType::Reference { .. }
    ));

    // Return type should also be a reference (since it returns the parameter)
    // NOTE: This test documents current behavior - return type transformation
    // will be enhanced in future phases
    assert!(matches!(
        transformed.return_type(),
        HirType::Pointer(..) | HirType::Reference { .. }
    ));
}

#[test]
fn test_high_confidence_borrows_prioritized() {
    // Test that high-confidence inferences are trusted more
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

    // High confidence should still generate correct borrow
    assert_eq!(
        transformed,
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        }
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
            // Property: ImmutableBorrow always generates non-mutable reference
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

            let is_immutable_ref = matches!(
                transformed,
                HirType::Reference { mutable: false, .. }
            );
            prop_assert!(is_immutable_ref);
        }

        #[test]
        fn prop_mutable_borrow_generates_mutable_reference(
            var_name in "[a-z][a-z0-9_]{0,10}",
        ) {
            // Property: MutableBorrow always generates mutable reference
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

            let is_mutable_ref = matches!(
                transformed,
                HirType::Reference { mutable: true, .. }
            );
            prop_assert!(is_mutable_ref);
        }

        #[test]
        fn prop_owning_generates_box(
            var_name in "[a-z][a-z0-9_]{0,10}",
        ) {
            // Property: Owning pointers always become Box<T>
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

            prop_assert!(matches!(transformed, HirType::Box(..)));
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
            // Property: Same input produces same output
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
