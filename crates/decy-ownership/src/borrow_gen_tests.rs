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
        HirType::Reference {
            mutable: false,
            ..
        }
    ));
    assert!(matches!(
        transformed2,
        HirType::Reference {
            mutable: false,
            ..
        }
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
        transformed, HirType::Int,
        "Non-pointer types should remain unchanged"
    );
}
