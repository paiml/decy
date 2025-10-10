//! Tests for lifetime annotation generation.

use super::*;
use decy_hir::{HirFunction, HirParameter, HirStatement, HirType};

#[test]
fn test_infer_lifetime_parameters() {
    // Test inferring lifetime parameters for function with reference parameter
    let func = HirFunction::new(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: false,
            },
        )],
    );

    let annotator = LifetimeAnnotator::new();
    let signature = annotator.annotate_function(&func);

    assert!(
        !signature.lifetimes.is_empty(),
        "Function with reference parameter should have lifetime parameters"
    );
    assert_eq!(
        signature.lifetimes[0].name, "'a",
        "First lifetime should be 'a"
    );
}

#[test]
fn test_generate_lifetime_syntax() {
    // Test generating lifetime syntax string
    let annotator = LifetimeAnnotator::new();

    let lifetimes = vec![LifetimeParam::standard(0)];
    let syntax = annotator.generate_lifetime_syntax(&lifetimes);
    assert_eq!(syntax, "<'a>", "Should generate <'a>");

    let lifetimes = vec![LifetimeParam::standard(0), LifetimeParam::standard(1)];
    let syntax = annotator.generate_lifetime_syntax(&lifetimes);
    assert_eq!(syntax, "<'a, 'b>", "Should generate <'a, 'b>");

    let lifetimes = vec![];
    let syntax = annotator.generate_lifetime_syntax(&lifetimes);
    assert_eq!(syntax, "", "Empty lifetimes should produce empty string");
}

#[test]
fn test_annotate_parameters() {
    // Test annotating parameters with lifetime
    let func = HirFunction::new(
        "test".to_string(),
        HirType::Void,
        vec![
            HirParameter::new(
                "data".to_string(),
                HirType::Reference {
                    inner: Box::new(HirType::Int),
                    mutable: false,
                },
            ),
            HirParameter::new("count".to_string(), HirType::Int),
        ],
    );

    let annotator = LifetimeAnnotator::new();
    let signature = annotator.annotate_function(&func);

    assert_eq!(signature.parameters.len(), 2);

    // First parameter (reference) should have lifetime annotation
    assert!(
        matches!(
            signature.parameters[0].param_type,
            AnnotatedType::Reference {
                lifetime: Some(_),
                ..
            }
        ),
        "Reference parameter should have lifetime"
    );

    // Second parameter (int) should be simple type
    assert!(
        matches!(signature.parameters[1].param_type, AnnotatedType::Simple(_)),
        "Non-reference parameter should be simple type"
    );
}

#[test]
fn test_annotate_return_type() {
    // Test annotating return type with lifetime
    let func = HirFunction::new(
        "test".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: false,
            },
        )],
    );

    let annotator = LifetimeAnnotator::new();
    let signature = annotator.annotate_function(&func);

    // Return type should have lifetime annotation
    assert!(
        matches!(
            signature.return_type,
            AnnotatedType::Reference {
                lifetime: Some(_),
                ..
            }
        ),
        "Reference return type should have lifetime"
    );
}

#[test]
fn test_validate_constraints() {
    // Test validating lifetime constraints
    let annotator = LifetimeAnnotator::new();

    // Valid: return lifetime exists in parameters
    let valid_signature = AnnotatedSignature {
        name: "test".to_string(),
        lifetimes: vec![LifetimeParam::standard(0)],
        parameters: vec![AnnotatedParameter {
            name: "data".to_string(),
            param_type: AnnotatedType::Reference {
                inner: Box::new(AnnotatedType::Simple(HirType::Int)),
                mutable: false,
                lifetime: Some(LifetimeParam::standard(0)),
            },
        }],
        return_type: AnnotatedType::Reference {
            inner: Box::new(AnnotatedType::Simple(HirType::Int)),
            mutable: false,
            lifetime: Some(LifetimeParam::standard(0)),
        },
    };

    assert!(
        annotator.validate_constraints(&valid_signature).is_ok(),
        "Valid signature should pass validation"
    );
}

#[test]
fn test_function_without_references() {
    // Test function with no references (should have no lifetime parameters)
    let func = HirFunction::new(
        "add".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("a".to_string(), HirType::Int),
            HirParameter::new("b".to_string(), HirType::Int),
        ],
    );

    let annotator = LifetimeAnnotator::new();
    let signature = annotator.annotate_function(&func);

    assert!(
        signature.lifetimes.is_empty(),
        "Function without references should have no lifetime parameters"
    );
}

#[test]
fn test_mutable_reference_parameter() {
    // Test handling mutable reference parameters
    let func = HirFunction::new(
        "modify".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: true,
            },
        )],
    );

    let annotator = LifetimeAnnotator::new();
    let signature = annotator.annotate_function(&func);

    assert!(!signature.lifetimes.is_empty());
    if let AnnotatedType::Reference {
        mutable, lifetime, ..
    } = &signature.parameters[0].param_type
    {
        assert!(*mutable, "Should preserve mutable flag");
        assert!(lifetime.is_some(), "Should have lifetime annotation");
    } else {
        panic!("Expected reference type");
    }
}

#[test]
fn test_function_lifetime_end_to_end() {
    // Integration test: complete lifetime annotation pipeline
    let func = HirFunction::new_with_body(
        "get_value".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
        vec![HirParameter::new(
            "container".to_string(),
            HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: false,
            },
        )],
        vec![HirStatement::Return(Some(
            decy_hir::HirExpression::Variable("container".to_string()),
        ))],
    );

    let annotator = LifetimeAnnotator::new();
    let signature = annotator.annotate_function(&func);

    // Should have one lifetime parameter
    assert_eq!(signature.lifetimes.len(), 1);
    assert_eq!(signature.lifetimes[0].name, "'a");

    // Parameter should use that lifetime
    if let AnnotatedType::Reference { lifetime, .. } = &signature.parameters[0].param_type {
        assert_eq!(lifetime.as_ref().unwrap().name, "'a");
    }

    // Return type should use that lifetime
    if let AnnotatedType::Reference { lifetime, .. } = &signature.return_type {
        assert_eq!(lifetime.as_ref().unwrap().name, "'a");
    }

    // Should validate successfully
    assert!(annotator.validate_constraints(&signature).is_ok());

    // Should generate proper syntax
    let syntax = annotator.generate_lifetime_syntax(&signature.lifetimes);
    assert_eq!(syntax, "<'a>");
}

#[test]
fn test_lifetime_param_standard() {
    // Test standard lifetime parameter generation
    assert_eq!(LifetimeParam::standard(0).name, "'a");
    assert_eq!(LifetimeParam::standard(1).name, "'b");
    assert_eq!(LifetimeParam::standard(2).name, "'c");
}

#[test]
fn test_nested_reference_types() {
    // Test handling nested reference types (rare but possible)
    let func = HirFunction::new(
        "test".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Reference {
                inner: Box::new(HirType::Reference {
                    inner: Box::new(HirType::Int),
                    mutable: false,
                }),
                mutable: false,
            },
        )],
    );

    let annotator = LifetimeAnnotator::new();
    let signature = annotator.annotate_function(&func);

    // Should handle nested references
    assert!(!signature.lifetimes.is_empty());
    assert_eq!(signature.parameters.len(), 1);
}

#[test]
fn test_multiple_reference_parameters() {
    // Test function with multiple reference parameters
    let func = HirFunction::new(
        "compare".to_string(),
        HirType::Int,
        vec![
            HirParameter::new(
                "a".to_string(),
                HirType::Reference {
                    inner: Box::new(HirType::Int),
                    mutable: false,
                },
            ),
            HirParameter::new(
                "b".to_string(),
                HirType::Reference {
                    inner: Box::new(HirType::Int),
                    mutable: false,
                },
            ),
        ],
    );

    let annotator = LifetimeAnnotator::new();
    let signature = annotator.annotate_function(&func);

    // For now, should use same lifetime for all parameters
    // Future: analyze dependencies to use different lifetimes
    assert!(!signature.lifetimes.is_empty());
    assert_eq!(signature.parameters.len(), 2);
}

#[test]
fn test_void_return_with_ref_params() {
    // Test function with reference parameters but void return
    let func = HirFunction::new(
        "process".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: true,
            },
        )],
    );

    let annotator = LifetimeAnnotator::new();
    let signature = annotator.annotate_function(&func);

    // Should have lifetime parameter for the reference
    assert!(!signature.lifetimes.is_empty());
    // But return type is simple (void)
    assert!(matches!(
        signature.return_type,
        AnnotatedType::Simple(HirType::Void)
    ));
}
