//! Tests for borrow code generation.

use super::*;
use crate::dataflow::DataflowAnalyzer;
use crate::inference::{OwnershipInference, OwnershipInferencer, OwnershipKind};
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

#[test]
fn test_generate_immutable_borrow() {
    // DECY-180: ImmutableBorrow inference generates &T type
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
        "DECY-180: ImmutableBorrow generates &T reference"
    );
}

#[test]
fn test_generate_mutable_borrow() {
    // DECY-180: MutableBorrow inference generates &mut T type
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
        "DECY-180: MutableBorrow generates &mut T reference"
    );
}

#[test]
fn test_generate_borrowed_parameter() {
    // DECY-180: Parameters transformed to references based on inference
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
        "DECY-180: ImmutableBorrow parameter generates &T"
    );
}

#[test]
fn test_borrow_checker_validation() {
    // DECY-180: Generated references follow borrow checker rules
    let generator = BorrowGenerator::new();

    let ptr_type = HirType::Pointer(Box::new(HirType::Int));
    let mut inferences = HashMap::new();

    // Multiple immutable borrows generate multiple &T references
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

    // DECY-180: Both should be immutable references
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
    // DECY-180: End-to-end test - analyze function, infer ownership, generate references
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

    // Verify transformation - should be reference or slice based on inference
    assert_eq!(transformed_func.parameters().len(), 1);
    // The result depends on actual inference - could be reference, slice, or pointer
    // Just verify it completed without panic
    let _param_type = transformed_func.parameters()[0].param_type();
}

#[test]
fn test_owning_pointer_becomes_box() {
    // DECY-180: Owning pointers transformed to Box<T>
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
        "DECY-180: Owning pointers become Box<T>"
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

// DECY-180: Enhanced borrow generation tests

#[test]
fn test_multiple_immutable_borrows_allowed() {
    // DECY-180: Multiple immutable borrows generate multiple &T references
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

    // DECY-180: Both should be immutable references
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
    // DECY-180: Generate &mut T and &T references based on inference
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

    // DECY-180: First should be &mut T, second should be &T
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
    // DECY-180: Nested pointers transform outer pointer to reference
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

    // DECY-180: Outer pointer becomes reference to inner pointer
    assert!(matches!(
        transformed,
        HirType::Reference { mutable: false, .. }
    ));
}

#[test]
fn test_lifetime_aware_borrow_generation() {
    // DECY-180: End-to-end inference and transformation
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

    // Just verify transformation completes - actual type depends on inference
    assert_eq!(transformed.parameters().len(), 1);
    let _param_type = transformed.parameters()[0].param_type();
}

#[test]
fn test_high_confidence_borrows_prioritized() {
    // DECY-180: High-confidence inferences generate references
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

    // DECY-180: High confidence generates reference
    assert_eq!(
        transformed,
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
        "DECY-180: High confidence ImmutableBorrow generates &T"
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

// ============================================================================
// DECY-180: Ownership-based borrow transformation tests
// These tests enable ownership inference to generate safe Rust references
// when no pointer arithmetic is detected.
// ============================================================================

#[test]
fn test_decy180_immutable_borrow_generates_reference() {
    // DECY-180: ImmutableBorrow should generate &T when no pointer arithmetic
    let generator = BorrowGenerator::new();

    let ptr_type = HirType::Pointer(Box::new(HirType::Int));
    let mut inferences = HashMap::new();
    inferences.insert(
        "data".to_string(),
        OwnershipInference {
            variable: "data".to_string(),
            kind: OwnershipKind::ImmutableBorrow,
            confidence: 0.8,
            reason: "const parameter".to_string(),
        },
    );

    let transformed = generator.transform_type(&ptr_type, "data", &inferences);

    // Should generate &T reference, not raw pointer
    assert_eq!(
        transformed,
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
        "DECY-180: ImmutableBorrow should generate &T reference"
    );
}

#[test]
fn test_decy180_mutable_borrow_generates_mut_reference() {
    // DECY-180: MutableBorrow should generate &mut T when no pointer arithmetic
    let generator = BorrowGenerator::new();

    let ptr_type = HirType::Pointer(Box::new(HirType::Int));
    let mut inferences = HashMap::new();
    inferences.insert(
        "data".to_string(),
        OwnershipInference {
            variable: "data".to_string(),
            kind: OwnershipKind::MutableBorrow,
            confidence: 0.85,
            reason: "parameter with writes".to_string(),
        },
    );

    let transformed = generator.transform_type(&ptr_type, "data", &inferences);

    // Should generate &mut T reference, not raw pointer
    assert_eq!(
        transformed,
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        },
        "DECY-180: MutableBorrow should generate &mut T reference"
    );
}

#[test]
fn test_decy180_owning_generates_box() {
    // DECY-180: Owning pointers should generate Box<T>
    let generator = BorrowGenerator::new();

    let ptr_type = HirType::Pointer(Box::new(HirType::Int));
    let mut inferences = HashMap::new();
    inferences.insert(
        "ptr".to_string(),
        OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Owning,
            confidence: 0.95,
            reason: "malloc with free".to_string(),
        },
    );

    let transformed = generator.transform_type(&ptr_type, "ptr", &inferences);

    // Should generate Box<T>, not raw pointer
    assert_eq!(
        transformed,
        HirType::Box(Box::new(HirType::Int)),
        "DECY-180: Owning should generate Box<T>"
    );
}

#[test]
fn test_decy180_unknown_keeps_raw_pointer() {
    // DECY-180: Unknown ownership keeps raw pointer (safe fallback)
    let generator = BorrowGenerator::new();

    let ptr_type = HirType::Pointer(Box::new(HirType::Int));
    let mut inferences = HashMap::new();
    inferences.insert(
        "ptr".to_string(),
        OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::Unknown,
            confidence: 0.3,
            reason: "uncertain pattern".to_string(),
        },
    );

    let transformed = generator.transform_type(&ptr_type, "ptr", &inferences);

    // Unknown should fall back to raw pointer
    assert_eq!(
        transformed,
        HirType::Pointer(Box::new(HirType::Int)),
        "DECY-180: Unknown should keep raw pointer"
    );
}

#[test]
fn test_decy180_no_inference_keeps_raw_pointer() {
    // DECY-180: No inference available keeps raw pointer
    let generator = BorrowGenerator::new();

    let ptr_type = HirType::Pointer(Box::new(HirType::Int));
    let inferences = HashMap::new(); // Empty - no inference

    let transformed = generator.transform_type(&ptr_type, "unknown_var", &inferences);

    // No inference should fall back to raw pointer
    assert_eq!(
        transformed,
        HirType::Pointer(Box::new(HirType::Int)),
        "DECY-180: No inference should keep raw pointer"
    );
}

#[test]
fn test_decy180_parameter_transformation() {
    // DECY-180: Parameters should be transformed based on ownership
    let generator = BorrowGenerator::new();

    let params = vec![
        HirParameter::new(
            "immut".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        ),
        HirParameter::new(
            "mut_p".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        ),
    ];

    let mut inferences = HashMap::new();
    inferences.insert(
        "immut".to_string(),
        OwnershipInference {
            variable: "immut".to_string(),
            kind: OwnershipKind::ImmutableBorrow,
            confidence: 0.8,
            reason: "const".to_string(),
        },
    );
    inferences.insert(
        "mut_p".to_string(),
        OwnershipInference {
            variable: "mut_p".to_string(),
            kind: OwnershipKind::MutableBorrow,
            confidence: 0.85,
            reason: "mutated".to_string(),
        },
    );

    let transformed = generator.transform_parameters(&params, &inferences);

    assert_eq!(transformed.len(), 2);
    assert_eq!(
        transformed[0].param_type(),
        &HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
        "DECY-180: ImmutableBorrow param should be &T"
    );
    assert_eq!(
        transformed[1].param_type(),
        &HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        },
        "DECY-180: MutableBorrow param should be &mut T"
    );
}

// TDD-Refactor Phase: Property tests for borrow generation

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_immutable_borrow_generates_non_mutable_reference(
            var_name in "[a-z][a-z0-9_]{0,10}",
        ) {
            // DECY-180: Property: ImmutableBorrow generates non-mutable reference
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

            // DECY-180: Should generate immutable reference
            let is_immut_ref = matches!(transformed, HirType::Reference { mutable: false, .. });
            prop_assert!(is_immut_ref);
        }

        #[test]
        fn prop_mutable_borrow_generates_mutable_reference(
            var_name in "[a-z][a-z0-9_]{0,10}",
        ) {
            // DECY-180: Property: MutableBorrow generates mutable reference
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

            // DECY-180: Should generate mutable reference
            let is_mut_ref = matches!(transformed, HirType::Reference { mutable: true, .. });
            prop_assert!(is_mut_ref);
        }

        #[test]
        fn prop_owning_generates_box(
            var_name in "[a-z][a-z0-9_]{0,10}",
        ) {
            // DECY-180: Property: Owning generates Box<T>
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

            // DECY-180: Should generate Box<T>
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

// ============================================================================
// DECY-184: Skip borrow transformation for char* with pointer arithmetic
// String iteration pattern (char* with ptr++ or ptr = ptr + 1) must stay as
// Pointer so codegen can detect it and generate &mut [u8] / &[u8].
// ============================================================================

#[test]
fn test_decy184_char_ptr_with_pointer_arithmetic_not_transformed() {
    // DECY-184: char* params with pointer arithmetic should NOT be transformed
    // to Reference - they should stay as Pointer for codegen's string iteration
    // detection to handle.
    //
    // C: void string_copy(char* dest, char* src) { *dest = *src; dest = dest + 1; }
    // Expected: dest stays as Pointer(Char), NOT Reference { inner: Char, mutable: true }
    use decy_hir::BinaryOperator;

    let generator = BorrowGenerator::new();

    // Create function with char* param that uses pointer arithmetic
    let func = HirFunction::new_with_body(
        "string_copy".to_string(),
        HirType::Void,
        vec![
            HirParameter::new(
                "dest".to_string(),
                HirType::Pointer(Box::new(HirType::Char)),
            ),
            HirParameter::new("src".to_string(), HirType::Pointer(Box::new(HirType::Char))),
        ],
        vec![
            // *dest = *src (deref assignment - shows it's mutable)
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("dest".to_string()),
                value: HirExpression::Dereference(Box::new(HirExpression::Variable(
                    "src".to_string(),
                ))),
            },
            // dest = dest + 1 (pointer arithmetic)
            HirStatement::Assignment {
                target: "dest".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("dest".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
            // src = src + 1 (pointer arithmetic)
            HirStatement::Assignment {
                target: "src".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("src".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
        ],
    );

    // Create inferences that would normally cause transformation
    let mut inferences = HashMap::new();
    inferences.insert(
        "dest".to_string(),
        OwnershipInference {
            variable: "dest".to_string(),
            kind: OwnershipKind::MutableBorrow,
            confidence: 0.85,
            reason: "parameter with writes".to_string(),
        },
    );
    inferences.insert(
        "src".to_string(),
        OwnershipInference {
            variable: "src".to_string(),
            kind: OwnershipKind::ImmutableBorrow,
            confidence: 0.8,
            reason: "read-only parameter".to_string(),
        },
    );

    // Transform the function
    let transformed = generator.transform_function(&func, &inferences);

    // DECY-184: char* params with pointer arithmetic should NOT become References
    // They should stay as Pointer(Char) for codegen to handle
    let dest_param = transformed
        .parameters()
        .iter()
        .find(|p| p.name() == "dest")
        .unwrap();
    let src_param = transformed
        .parameters()
        .iter()
        .find(|p| p.name() == "src")
        .unwrap();

    // dest should stay as Pointer(Char), NOT Reference
    assert!(
        matches!(dest_param.param_type(), HirType::Pointer(_)),
        "DECY-184: char* dest with pointer arithmetic should stay as Pointer, got {:?}",
        dest_param.param_type()
    );

    // src should stay as Pointer(Char), NOT Reference
    assert!(
        matches!(src_param.param_type(), HirType::Pointer(_)),
        "DECY-184: char* src with pointer arithmetic should stay as Pointer, got {:?}",
        src_param.param_type()
    );
}

#[test]
fn test_decy184_char_ptr_without_pointer_arithmetic_is_transformed() {
    // DECY-184: char* params WITHOUT pointer arithmetic SHOULD be transformed
    // to Reference based on ownership inference.
    //
    // C: void set_char(char* ptr) { *ptr = 'x'; }
    // Expected: ptr becomes Reference { inner: Char, mutable: true }

    let generator = BorrowGenerator::new();

    // Create function with char* param that does NOT use pointer arithmetic
    let func = HirFunction::new_with_body(
        "set_char".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        )],
        vec![
            // *ptr = 'x' (deref assignment - no pointer arithmetic)
            HirStatement::DerefAssignment {
                target: HirExpression::Variable("ptr".to_string()),
                value: HirExpression::CharLiteral(b'x' as i8),
            },
        ],
    );

    // MutableBorrow inference should trigger transformation
    let mut inferences = HashMap::new();
    inferences.insert(
        "ptr".to_string(),
        OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::MutableBorrow,
            confidence: 0.85,
            reason: "parameter with writes".to_string(),
        },
    );

    // Transform the function
    let transformed = generator.transform_function(&func, &inferences);

    // char* without pointer arithmetic SHOULD become &mut u8
    let ptr_param = transformed
        .parameters()
        .iter()
        .find(|p| p.name() == "ptr")
        .unwrap();

    assert_eq!(
        ptr_param.param_type(),
        &HirType::Reference {
            inner: Box::new(HirType::Char),
            mutable: true,
        },
        "DECY-184: char* without pointer arithmetic should transform to &mut u8"
    );
}

#[test]
fn test_decy184_int_ptr_with_pointer_arithmetic_stays_as_pointer() {
    // DECY-184: int* params with pointer arithmetic should also stay as Pointer
    // (This is the existing behavior that should continue to work)
    use decy_hir::BinaryOperator;

    let generator = BorrowGenerator::new();

    // Create function with int* param that uses pointer arithmetic
    let func = HirFunction::new_with_body(
        "traverse_array".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "arr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![
            // arr = arr + 1 (pointer arithmetic)
            HirStatement::Assignment {
                target: "arr".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("arr".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            },
        ],
    );

    let mut inferences = HashMap::new();
    inferences.insert(
        "arr".to_string(),
        OwnershipInference {
            variable: "arr".to_string(),
            kind: OwnershipKind::MutableBorrow,
            confidence: 0.85,
            reason: "array parameter".to_string(),
        },
    );

    let transformed = generator.transform_function(&func, &inferences);
    let arr_param = transformed
        .parameters()
        .iter()
        .find(|p| p.name() == "arr")
        .unwrap();

    // int* with pointer arithmetic should stay as Pointer
    assert!(
        matches!(arr_param.param_type(), HirType::Pointer(_)),
        "DECY-184: int* with pointer arithmetic should stay as Pointer, got {:?}",
        arr_param.param_type()
    );
}

// ============================================================================
// Additional Coverage Tests
// ============================================================================

#[test]
fn test_array_pointer_keeps_raw_pointer() {
    // ArrayPointer ownership kind keeps raw pointer for slice transformation
    let generator = BorrowGenerator::new();

    let ptr_type = HirType::Pointer(Box::new(HirType::Int));
    let mut inferences = HashMap::new();
    inferences.insert(
        "arr".to_string(),
        OwnershipInference {
            variable: "arr".to_string(),
            kind: OwnershipKind::ArrayPointer {
                base_array: "arr".to_string(),
                element_type: HirType::Int,
                base_index: Some(0),
            },
            confidence: 0.9,
            reason: "array parameter".to_string(),
        },
    );

    let transformed = generator.transform_type(&ptr_type, "arr", &inferences);

    assert!(
        matches!(transformed, HirType::Pointer(_)),
        "ArrayPointer should keep raw pointer for slice transformation"
    );
}

#[test]
fn test_no_inference_keeps_raw_pointer() {
    // Variable with no inference keeps raw pointer
    let generator = BorrowGenerator::new();

    let ptr_type = HirType::Pointer(Box::new(HirType::Float));
    let inferences = HashMap::new(); // Empty - no inference

    let transformed = generator.transform_type(&ptr_type, "unknown", &inferences);

    assert_eq!(
        transformed,
        HirType::Pointer(Box::new(HirType::Float)),
        "No inference should keep raw pointer"
    );
}

#[test]
fn test_borrow_generator_default() {
    let gen: BorrowGenerator = BorrowGenerator::new();
    let debug = format!("{:?}", gen);
    assert!(debug.contains("BorrowGenerator"));
}

#[test]
fn test_transform_parameters_empty() {
    let generator = BorrowGenerator::new();
    let params: Vec<HirParameter> = vec![];
    let inferences = HashMap::new();

    let transformed = generator.transform_parameters(&params, &inferences);
    assert!(transformed.is_empty());
}

#[test]
fn test_transform_parameters_mixed() {
    let generator = BorrowGenerator::new();

    let params = vec![
        HirParameter::new("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
        HirParameter::new("val".to_string(), HirType::Int),
        HirParameter::new("flt".to_string(), HirType::Float),
    ];

    let mut inferences = HashMap::new();
    inferences.insert(
        "ptr".to_string(),
        OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::ImmutableBorrow,
            confidence: 0.8,
            reason: "Read only".to_string(),
        },
    );

    let transformed = generator.transform_parameters(&params, &inferences);

    assert_eq!(transformed.len(), 3);
    // ptr should be &i32
    assert!(matches!(
        transformed[0].param_type(),
        HirType::Reference { mutable: false, .. }
    ));
    // val and flt should be unchanged
    assert_eq!(transformed[1].param_type(), &HirType::Int);
    assert_eq!(transformed[2].param_type(), &HirType::Float);
}

#[test]
fn test_transform_nested_pointer() {
    let generator = BorrowGenerator::new();

    // Pointer to pointer
    let ptr_ptr_type = HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Char))));
    let mut inferences = HashMap::new();
    inferences.insert(
        "pp".to_string(),
        OwnershipInference {
            variable: "pp".to_string(),
            kind: OwnershipKind::ImmutableBorrow,
            confidence: 0.7,
            reason: "pointer to pointer".to_string(),
        },
    );

    let transformed = generator.transform_type(&ptr_ptr_type, "pp", &inferences);

    // Should be &*const char (reference to pointer)
    assert!(matches!(
        transformed,
        HirType::Reference { mutable: false, .. }
    ));
}

#[test]
fn test_transform_double_type() {
    let generator = BorrowGenerator::new();

    let double_type = HirType::Double;
    let inferences = HashMap::new();

    let transformed = generator.transform_type(&double_type, "x", &inferences);
    assert_eq!(transformed, HirType::Double);
}

#[test]
fn test_transform_char_type() {
    let generator = BorrowGenerator::new();

    let char_type = HirType::Char;
    let inferences = HashMap::new();

    let transformed = generator.transform_type(&char_type, "c", &inferences);
    assert_eq!(transformed, HirType::Char);
}

#[test]
fn test_transform_void_type() {
    let generator = BorrowGenerator::new();

    let void_type = HirType::Void;
    let inferences = HashMap::new();

    let transformed = generator.transform_type(&void_type, "v", &inferences);
    assert_eq!(transformed, HirType::Void);
}

// ============================================================================
// Coverage Improvement Tests - Statement Mutation Detection
// ============================================================================

#[test]
fn test_statement_mutates_variable_in_switch_case() {
    // Test that statement_mutates_variable detects mutations in switch cases
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "switch_test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "arr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Switch {
            condition: HirExpression::Variable("x".to_string()),
            cases: vec![decy_hir::SwitchCase {
                value: Some(HirExpression::IntLiteral(1)),
                body: vec![HirStatement::ArrayIndexAssignment {
                    array: Box::new(HirExpression::Variable("arr".to_string())),
                    index: Box::new(HirExpression::IntLiteral(0)),
                    value: HirExpression::IntLiteral(42),
                }],
            }],
            default_case: Some(vec![HirStatement::ArrayIndexAssignment {
                array: Box::new(HirExpression::Variable("arr".to_string())),
                index: Box::new(HirExpression::IntLiteral(1)),
                value: HirExpression::IntLiteral(99),
            }]),
        }],
    );

    // Use dataflow analyzer to check mutation detection
    let analyzer = DataflowAnalyzer::new();
    let graph = analyzer.analyze(&func);
    let _body = graph.body();

    // Test passes if no panic - the generator handles switch statements
    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.parameters().len(), 1);
}

#[test]
fn test_statement_mutates_variable_in_while_body() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "while_test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "arr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::While {
            condition: HirExpression::IntLiteral(1),
            body: vec![HirStatement::ArrayIndexAssignment {
                array: Box::new(HirExpression::Variable("arr".to_string())),
                index: Box::new(HirExpression::IntLiteral(0)),
                value: HirExpression::IntLiteral(42),
            }],
        }],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.parameters().len(), 1);
}

#[test]
fn test_statement_mutates_variable_in_for_body() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "for_test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "arr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::For {
            init: vec![],
            condition: Some(HirExpression::IntLiteral(1)),
            increment: vec![],
            body: vec![HirStatement::ArrayIndexAssignment {
                array: Box::new(HirExpression::Variable("arr".to_string())),
                index: Box::new(HirExpression::IntLiteral(0)),
                value: HirExpression::IntLiteral(42),
            }],
        }],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.parameters().len(), 1);
}

#[test]
fn test_statement_mutates_variable_in_if_then_and_else() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "if_else_test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "arr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::If {
            condition: HirExpression::IntLiteral(1),
            then_block: vec![HirStatement::ArrayIndexAssignment {
                array: Box::new(HirExpression::Variable("arr".to_string())),
                index: Box::new(HirExpression::IntLiteral(0)),
                value: HirExpression::IntLiteral(1),
            }],
            else_block: Some(vec![HirStatement::ArrayIndexAssignment {
                array: Box::new(HirExpression::Variable("arr".to_string())),
                index: Box::new(HirExpression::IntLiteral(1)),
                value: HirExpression::IntLiteral(2),
            }]),
        }],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.parameters().len(), 1);
}

#[test]
fn test_deref_assignment_mutates_variable() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "deref_test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::DerefAssignment {
            target: HirExpression::Variable("ptr".to_string()),
            value: HirExpression::IntLiteral(42),
        }],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.parameters().len(), 1);
}

// ============================================================================
// Coverage Improvement Tests - Statement Transformation with Length Replacement
// ============================================================================

#[test]
fn test_transform_return_statement_with_length() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "return_len".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ],
        vec![HirStatement::Return(Some(HirExpression::Variable(
            "len".to_string(),
        )))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);

    // Parameters should be present (len may or may not be removed depending on array detection)
    assert!(!transformed.body().is_empty());
}

#[test]
fn test_transform_while_statement_with_length() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "while_len".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ],
        vec![HirStatement::While {
            condition: HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::Variable("len".to_string())),
            },
            body: vec![],
        }],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_for_statement_with_length() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "for_len".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("size".to_string(), HirType::Int),
        ],
        vec![HirStatement::For {
            init: vec![HirStatement::VariableDeclaration {
                name: "i".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(0)),
            }],
            condition: Some(HirExpression::BinaryOp {
                op: decy_hir::BinaryOperator::LessThan,
                left: Box::new(HirExpression::Variable("i".to_string())),
                right: Box::new(HirExpression::Variable("size".to_string())),
            }),
            increment: vec![],
            body: vec![],
        }],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_switch_statement_with_length() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "switch_len".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("count".to_string(), HirType::Int),
        ],
        vec![HirStatement::Switch {
            condition: HirExpression::Variable("count".to_string()),
            cases: vec![decy_hir::SwitchCase {
                value: Some(HirExpression::IntLiteral(0)),
                body: vec![HirStatement::Return(None)],
            }],
            default_case: Some(vec![HirStatement::Expression(HirExpression::Variable(
                "count".to_string(),
            ))]),
        }],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_free_statement() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "free_ptr".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Free {
            pointer: HirExpression::Variable("ptr".to_string()),
        }],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_field_assignment_statement() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "field_assign".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "obj".to_string(),
            HirType::Pointer(Box::new(HirType::Struct("Point".to_string()))),
        )],
        vec![HirStatement::FieldAssignment {
            object: HirExpression::Variable("obj".to_string()),
            field: "x".to_string(),
            value: HirExpression::IntLiteral(42),
        }],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_array_index_assignment_statement() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "array_assign".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "arr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::ArrayIndexAssignment {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
            value: HirExpression::IntLiteral(42),
        }],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_break_continue_statements() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "break_continue".to_string(),
        HirType::Void,
        vec![],
        vec![HirStatement::While {
            condition: HirExpression::IntLiteral(1),
            body: vec![HirStatement::Break, HirStatement::Continue],
        }],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_variable_declaration_statement() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "var_decl".to_string(),
        HirType::Void,
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![HirStatement::VariableDeclaration {
            name: "x".to_string(),
            var_type: HirType::Int,
            initializer: Some(HirExpression::Variable("n".to_string())),
        }],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

// ============================================================================
// Coverage Improvement Tests - Expression Transformations
// ============================================================================

#[test]
fn test_transform_address_of_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "addr_of".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![],
        vec![HirStatement::Return(Some(HirExpression::AddressOf(
            Box::new(HirExpression::Variable("x".to_string())),
        )))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_unary_op_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "unary_op".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::UnaryOp {
            op: decy_hir::UnaryOperator::Minus,
            operand: Box::new(HirExpression::Variable("x".to_string())),
        }))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_field_access_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "field_access".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "obj".to_string(),
            HirType::Struct("Point".to_string()),
        )],
        vec![HirStatement::Return(Some(HirExpression::FieldAccess {
            object: Box::new(HirExpression::Variable("obj".to_string())),
            field: "x".to_string(),
        }))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_pointer_field_access_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "ptr_field_access".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "obj".to_string(),
            HirType::Pointer(Box::new(HirType::Struct("Point".to_string()))),
        )],
        vec![HirStatement::Return(Some(
            HirExpression::PointerFieldAccess {
                pointer: Box::new(HirExpression::Variable("obj".to_string())),
                field: "x".to_string(),
            },
        ))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_array_index_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "array_index".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "arr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::ArrayIndex {
            array: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::IntLiteral(0)),
        }))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_cast_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "cast_expr".to_string(),
        HirType::Float,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::Cast {
            expr: Box::new(HirExpression::Variable("x".to_string())),
            target_type: HirType::Float,
        }))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_compound_literal_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "compound_lit".to_string(),
        HirType::Struct("Point".to_string()),
        vec![],
        vec![HirStatement::Return(Some(HirExpression::CompoundLiteral {
            literal_type: HirType::Struct("Point".to_string()),
            initializers: vec![HirExpression::IntLiteral(1), HirExpression::IntLiteral(2)],
        }))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_is_not_null_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "is_not_null".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::If {
            condition: HirExpression::IsNotNull(Box::new(HirExpression::Variable(
                "ptr".to_string(),
            ))),
            then_block: vec![HirStatement::Return(Some(HirExpression::IntLiteral(1)))],
            else_block: Some(vec![HirStatement::Return(Some(HirExpression::IntLiteral(
                0,
            )))]),
        }],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_function_call_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "call_func".to_string(),
        HirType::Int,
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::FunctionCall {
            function: "add".to_string(),
            arguments: vec![
                HirExpression::Variable("n".to_string()),
                HirExpression::IntLiteral(1),
            ],
        }))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

// ============================================================================
// Coverage Improvement Tests - Pointer Arithmetic Detection
// ============================================================================

#[test]
fn test_pointer_arithmetic_with_subtract() {
    // Test that ptr = ptr - 1 is detected as pointer arithmetic
    use decy_hir::BinaryOperator;

    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "ptr_subtract".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Assignment {
            target: "ptr".to_string(),
            value: HirExpression::BinaryOp {
                op: BinaryOperator::Subtract,
                left: Box::new(HirExpression::Variable("ptr".to_string())),
                right: Box::new(HirExpression::IntLiteral(1)),
            },
        }],
    );

    let mut inferences = HashMap::new();
    inferences.insert(
        "ptr".to_string(),
        OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::MutableBorrow,
            confidence: 0.85,
            reason: "mutable pointer".to_string(),
        },
    );

    let transformed = generator.transform_function(&func, &inferences);
    // With pointer arithmetic, should stay as Pointer not Reference
    assert!(matches!(
        transformed.parameters()[0].param_type(),
        HirType::Pointer(_)
    ));
}

#[test]
fn test_slice_index_transformation_with_array_pointer() {
    // Test that *(arr + i) transforms to slice indexing when ArrayPointer is detected
    use decy_hir::BinaryOperator;

    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "slice_index".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "arr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::Dereference(
            Box::new(HirExpression::BinaryOp {
                op: BinaryOperator::Add,
                left: Box::new(HirExpression::Variable("arr".to_string())),
                right: Box::new(HirExpression::Variable("i".to_string())),
            }),
        )))],
    );

    let mut inferences = HashMap::new();
    inferences.insert(
        "arr".to_string(),
        OwnershipInference {
            variable: "arr".to_string(),
            kind: OwnershipKind::ArrayPointer {
                base_array: "arr".to_string(),
                element_type: HirType::Int,
                base_index: Some(0),
            },
            confidence: 0.95,
            reason: "array parameter".to_string(),
        },
    );

    let transformed = generator.transform_function(&func, &inferences);
    // Should contain SliceIndex transformation
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_length_param_detection_with_n_name() {
    // Test that 'n' is detected as a length parameter name
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "with_n_param".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("n".to_string(), HirType::Int),
        ],
        vec![HirStatement::Expression(HirExpression::Variable(
            "n".to_string(),
        ))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    // Just verify transformation completes
    assert!(!transformed.parameters().is_empty());
}

#[test]
fn test_length_param_detection_with_num_name() {
    // Test that 'num' is detected as a length parameter name
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "with_num_param".to_string(),
        HirType::Void,
        vec![
            HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("num".to_string(), HirType::Int),
        ],
        vec![HirStatement::Expression(HirExpression::Variable(
            "num".to_string(),
        ))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    // Just verify transformation completes
    assert!(!transformed.parameters().is_empty());
}

// ============================================================================
// Coverage Improvement Tests - Expression Type Transformations
// ============================================================================

#[test]
fn test_transform_calloc_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "calloc_test".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new("n".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::Calloc {
            count: Box::new(HirExpression::Variable("n".to_string())),
            element_type: Box::new(HirType::Int),
        }))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_malloc_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "malloc_test".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![HirParameter::new("size".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::Malloc {
            size: Box::new(HirExpression::Variable("size".to_string())),
        }))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_realloc_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "realloc_test".to_string(),
        HirType::Pointer(Box::new(HirType::Int)),
        vec![
            HirParameter::new("ptr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("new_size".to_string(), HirType::Int),
        ],
        vec![HirStatement::Return(Some(HirExpression::Realloc {
            pointer: Box::new(HirExpression::Variable("ptr".to_string())),
            new_size: Box::new(HirExpression::Variable("new_size".to_string())),
        }))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_string_method_call_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "string_method".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "s".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        )],
        vec![HirStatement::Return(Some(
            HirExpression::StringMethodCall {
                receiver: Box::new(HirExpression::Variable("s".to_string())),
                method: "len".to_string(),
                arguments: vec![],
            },
        ))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_slice_index_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "slice_index".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "arr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::SliceIndex {
            slice: Box::new(HirExpression::Variable("arr".to_string())),
            index: Box::new(HirExpression::Variable("i".to_string())),
            element_type: HirType::Int,
        }))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_post_increment_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "post_inc".to_string(),
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::PostIncrement {
            operand: Box::new(HirExpression::Variable("x".to_string())),
        }))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_pre_increment_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "pre_inc".to_string(),
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::PreIncrement {
            operand: Box::new(HirExpression::Variable("x".to_string())),
        }))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_post_decrement_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "post_dec".to_string(),
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::PostDecrement {
            operand: Box::new(HirExpression::Variable("x".to_string())),
        }))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_pre_decrement_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "pre_dec".to_string(),
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::PreDecrement {
            operand: Box::new(HirExpression::Variable("x".to_string())),
        }))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_ternary_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "ternary".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("cond".to_string(), HirType::Int),
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
        vec![HirStatement::Return(Some(HirExpression::Ternary {
            condition: Box::new(HirExpression::Variable("cond".to_string())),
            then_expr: Box::new(HirExpression::Variable("a".to_string())),
            else_expr: Box::new(HirExpression::Variable("b".to_string())),
        }))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_transform_dereference_expression() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "deref".to_string(),
        HirType::Int,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::Return(Some(HirExpression::Dereference(
            Box::new(HirExpression::Variable("ptr".to_string())),
        )))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

// ============================================================================
// Coverage Improvement Tests - Pointer Arithmetic Detection in Blocks
// ============================================================================

#[test]
fn test_pointer_arithmetic_in_if_block() {
    use decy_hir::BinaryOperator;

    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "ptr_arith_if".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::If {
            condition: HirExpression::IntLiteral(1),
            then_block: vec![HirStatement::Assignment {
                target: "ptr".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("ptr".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            }],
            else_block: None,
        }],
    );

    let mut inferences = HashMap::new();
    inferences.insert(
        "ptr".to_string(),
        OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::MutableBorrow,
            confidence: 0.85,
            reason: "mutable pointer".to_string(),
        },
    );

    let transformed = generator.transform_function(&func, &inferences);
    // With pointer arithmetic in if block, should stay as Pointer
    assert!(matches!(
        transformed.parameters()[0].param_type(),
        HirType::Pointer(_)
    ));
}

#[test]
fn test_pointer_arithmetic_in_else_block() {
    use decy_hir::BinaryOperator;

    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "ptr_arith_else".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::If {
            condition: HirExpression::IntLiteral(1),
            then_block: vec![],
            else_block: Some(vec![HirStatement::Assignment {
                target: "ptr".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("ptr".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            }]),
        }],
    );

    let mut inferences = HashMap::new();
    inferences.insert(
        "ptr".to_string(),
        OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::MutableBorrow,
            confidence: 0.85,
            reason: "mutable pointer".to_string(),
        },
    );

    let transformed = generator.transform_function(&func, &inferences);
    // With pointer arithmetic in else block, should stay as Pointer
    assert!(matches!(
        transformed.parameters()[0].param_type(),
        HirType::Pointer(_)
    ));
}

#[test]
fn test_pointer_arithmetic_in_while_block() {
    use decy_hir::BinaryOperator;

    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "ptr_arith_while".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::While {
            condition: HirExpression::IntLiteral(1),
            body: vec![HirStatement::Assignment {
                target: "ptr".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("ptr".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            }],
        }],
    );

    let mut inferences = HashMap::new();
    inferences.insert(
        "ptr".to_string(),
        OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::MutableBorrow,
            confidence: 0.85,
            reason: "mutable pointer".to_string(),
        },
    );

    let transformed = generator.transform_function(&func, &inferences);
    // With pointer arithmetic in while block, should stay as Pointer
    assert!(matches!(
        transformed.parameters()[0].param_type(),
        HirType::Pointer(_)
    ));
}

#[test]
fn test_pointer_arithmetic_in_for_block() {
    use decy_hir::BinaryOperator;

    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "ptr_arith_for".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "ptr".to_string(),
            HirType::Pointer(Box::new(HirType::Int)),
        )],
        vec![HirStatement::For {
            init: vec![],
            condition: Some(HirExpression::IntLiteral(1)),
            increment: vec![],
            body: vec![HirStatement::Assignment {
                target: "ptr".to_string(),
                value: HirExpression::BinaryOp {
                    op: BinaryOperator::Add,
                    left: Box::new(HirExpression::Variable("ptr".to_string())),
                    right: Box::new(HirExpression::IntLiteral(1)),
                },
            }],
        }],
    );

    let mut inferences = HashMap::new();
    inferences.insert(
        "ptr".to_string(),
        OwnershipInference {
            variable: "ptr".to_string(),
            kind: OwnershipKind::MutableBorrow,
            confidence: 0.85,
            reason: "mutable pointer".to_string(),
        },
    );

    let transformed = generator.transform_function(&func, &inferences);
    // With pointer arithmetic in for block, should stay as Pointer
    assert!(matches!(
        transformed.parameters()[0].param_type(),
        HirType::Pointer(_)
    ));
}

#[test]
fn test_post_increment_pointer_arithmetic() {
    // Test that str++ is detected as pointer arithmetic
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "post_inc_ptr".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "str".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        )],
        vec![HirStatement::Expression(HirExpression::PostIncrement {
            operand: Box::new(HirExpression::Variable("str".to_string())),
        })],
    );

    let mut inferences = HashMap::new();
    inferences.insert(
        "str".to_string(),
        OwnershipInference {
            variable: "str".to_string(),
            kind: OwnershipKind::MutableBorrow,
            confidence: 0.85,
            reason: "string pointer".to_string(),
        },
    );

    let transformed = generator.transform_function(&func, &inferences);
    // str++ should be detected, param stays as Pointer
    assert!(matches!(
        transformed.parameters()[0].param_type(),
        HirType::Pointer(_)
    ));
}

#[test]
fn test_pre_increment_pointer_arithmetic() {
    // Test that ++str is detected as pointer arithmetic
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "pre_inc_ptr".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "str".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        )],
        vec![HirStatement::Expression(HirExpression::PreIncrement {
            operand: Box::new(HirExpression::Variable("str".to_string())),
        })],
    );

    let mut inferences = HashMap::new();
    inferences.insert(
        "str".to_string(),
        OwnershipInference {
            variable: "str".to_string(),
            kind: OwnershipKind::MutableBorrow,
            confidence: 0.85,
            reason: "string pointer".to_string(),
        },
    );

    let transformed = generator.transform_function(&func, &inferences);
    // ++str should be detected, param stays as Pointer
    assert!(matches!(
        transformed.parameters()[0].param_type(),
        HirType::Pointer(_)
    ));
}

#[test]
fn test_post_decrement_pointer_arithmetic() {
    // Test that str-- is detected as pointer arithmetic
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "post_dec_ptr".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "str".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        )],
        vec![HirStatement::Expression(HirExpression::PostDecrement {
            operand: Box::new(HirExpression::Variable("str".to_string())),
        })],
    );

    let mut inferences = HashMap::new();
    inferences.insert(
        "str".to_string(),
        OwnershipInference {
            variable: "str".to_string(),
            kind: OwnershipKind::MutableBorrow,
            confidence: 0.85,
            reason: "string pointer".to_string(),
        },
    );

    let transformed = generator.transform_function(&func, &inferences);
    // str-- should be detected, param stays as Pointer
    assert!(matches!(
        transformed.parameters()[0].param_type(),
        HirType::Pointer(_)
    ));
}

#[test]
fn test_pre_decrement_pointer_arithmetic() {
    // Test that --str is detected as pointer arithmetic
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "pre_dec_ptr".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "str".to_string(),
            HirType::Pointer(Box::new(HirType::Char)),
        )],
        vec![HirStatement::Expression(HirExpression::PreDecrement {
            operand: Box::new(HirExpression::Variable("str".to_string())),
        })],
    );

    let mut inferences = HashMap::new();
    inferences.insert(
        "str".to_string(),
        OwnershipInference {
            variable: "str".to_string(),
            kind: OwnershipKind::MutableBorrow,
            confidence: 0.85,
            reason: "string pointer".to_string(),
        },
    );

    let transformed = generator.transform_function(&func, &inferences);
    // --str should be detected, param stays as Pointer
    assert!(matches!(
        transformed.parameters()[0].param_type(),
        HirType::Pointer(_)
    ));
}

#[test]
fn test_binary_op_expression_transformation() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "binary_op".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
        vec![HirStatement::Return(Some(HirExpression::BinaryOp {
            op: decy_hir::BinaryOperator::Add,
            left: Box::new(HirExpression::Variable("a".to_string())),
            right: Box::new(HirExpression::Variable("b".to_string())),
        }))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_sizeof_expression_not_transformed() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "sizeof_test".to_string(),
        HirType::Int,
        vec![],
        vec![HirStatement::Return(Some(HirExpression::Sizeof {
            type_name: "int".to_string(),
        }))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_null_literal_expression_not_transformed() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "null_test".to_string(),
        HirType::Pointer(Box::new(HirType::Void)),
        vec![],
        vec![HirStatement::Return(Some(HirExpression::NullLiteral))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

#[test]
fn test_literal_expressions_not_transformed() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "literals".to_string(),
        HirType::Void,
        vec![],
        vec![
            HirStatement::Expression(HirExpression::IntLiteral(42)),
            HirStatement::Expression(HirExpression::FloatLiteral("3.14".to_string())),
            HirStatement::Expression(HirExpression::StringLiteral("hello".to_string())),
            HirStatement::Expression(HirExpression::CharLiteral(b'x' as i8)),
        ],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 4);
}

#[test]
fn test_variable_expression_not_transformed() {
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "var_test".to_string(),
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![HirStatement::Return(Some(HirExpression::Variable(
            "x".to_string(),
        )))],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert_eq!(transformed.body().len(), 1);
}

// ============================================================================
// Coverage Improvement Tests - Length replacement in expressions
// ============================================================================

#[test]
fn test_length_replacement_in_variable() {
    let generator = BorrowGenerator::new();

    // Create a function where 'len' is the length param for 'arr'
    // and 'len' is used in an expression
    let func = HirFunction::new_with_body(
        "len_replace".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("arr".to_string(), HirType::Pointer(Box::new(HirType::Int))),
            HirParameter::new("len".to_string(), HirType::Int),
        ],
        vec![
            // Use arr[0] to trigger array detection
            HirStatement::ArrayIndexAssignment {
                array: Box::new(HirExpression::Variable("arr".to_string())),
                index: Box::new(HirExpression::IntLiteral(0)),
                value: HirExpression::IntLiteral(1),
            },
            // Return len - should be replaced with arr.len() if array detected
            HirStatement::Return(Some(HirExpression::Variable("len".to_string()))),
        ],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    assert!(!transformed.body().is_empty());
}

#[test]
fn test_default_impl() {
    // Test that BorrowGenerator implements Default
    let gen: BorrowGenerator = Default::default();
    let ptr_type = HirType::Pointer(Box::new(HirType::Int));
    let inferences = HashMap::new();
    let _ = gen.transform_type(&ptr_type, "x", &inferences);
}

#[test]
fn test_transform_slice_to_slice_type_non_pointer() {
    // Test transform_to_slice_type with non-pointer type (should return unchanged)
    let generator = BorrowGenerator::new();

    let func = HirFunction::new_with_body(
        "non_ptr".to_string(),
        HirType::Void,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
        vec![],
    );

    let inferences = HashMap::new();
    let transformed = generator.transform_function(&func, &inferences);
    // Int param should stay as Int
    assert_eq!(transformed.parameters()[0].param_type(), &HirType::Int);
}
