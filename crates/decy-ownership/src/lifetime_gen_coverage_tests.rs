//! Coverage tests for lifetime_gen.rs - targeting uncovered branches.

use crate::lifetime_gen::*;
use decy_hir::{HirExpression, HirFunction, HirParameter, HirStatement, HirType};

// ============================================================================
// LifetimeParam coverage
// ============================================================================

#[test]
fn lifetime_param_new_creates_named_param() {
    let param = LifetimeParam::new("'x".to_string());
    assert_eq!(param.name, "'x");
}

#[test]
fn lifetime_param_new_custom_name() {
    let param = LifetimeParam::new("'custom_lifetime".to_string());
    assert_eq!(param.name, "'custom_lifetime");
}

#[test]
fn lifetime_param_standard_generates_sequential_names() {
    // Standard already tested but ensure multiple indices work
    let d = LifetimeParam::standard(3);
    assert_eq!(d.name, "'d");
    let e = LifetimeParam::standard(4);
    assert_eq!(e.name, "'e");
}

#[test]
fn lifetime_param_equality() {
    let a1 = LifetimeParam::new("'a".to_string());
    let a2 = LifetimeParam::standard(0);
    assert_eq!(a1, a2);
}

#[test]
fn lifetime_param_hash_consistency() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(LifetimeParam::standard(0));
    // Inserting same value should not increase size
    set.insert(LifetimeParam::new("'a".to_string()));
    assert_eq!(set.len(), 1);
}

// ============================================================================
// LifetimeAnnotator::default coverage
// ============================================================================

#[test]
fn lifetime_annotator_default_trait() {
    let annotator: LifetimeAnnotator = Default::default();
    // Verify it works by annotating a simple function
    let func = HirFunction::new(
        "test".to_string(),
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
    );
    let sig = annotator.annotate_function(&func);
    assert_eq!(sig.name, "test");
    assert!(sig.lifetimes.is_empty());
}

// ============================================================================
// validate_constraints - error path coverage
// ============================================================================

#[test]
fn validate_constraints_returns_error_when_return_lifetime_not_in_params_or_signature() {
    let annotator = LifetimeAnnotator::new();

    // Construct a signature where return type has a lifetime 'b,
    // but neither parameters nor signature lifetimes contain 'b.
    let sig = AnnotatedSignature {
        name: "bad_func".to_string(),
        lifetimes: vec![], // No lifetimes declared
        parameters: vec![AnnotatedParameter {
            name: "x".to_string(),
            param_type: AnnotatedType::Simple(HirType::Int), // No lifetime in params
        }],
        return_type: AnnotatedType::Reference {
            inner: Box::new(AnnotatedType::Simple(HirType::Int)),
            mutable: false,
            lifetime: Some(LifetimeParam::new("'b".to_string())),
        },
    };

    let result = annotator.validate_constraints(&sig);
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("'b"),
        "Error should mention the missing lifetime"
    );
    assert!(
        err_msg.contains("not found in parameters"),
        "Error should explain the issue"
    );
}

#[test]
fn validate_constraints_ok_when_return_lifetime_in_signature_lifetimes() {
    let annotator = LifetimeAnnotator::new();

    // Return lifetime 'a is in signature.lifetimes even though no param has it
    let sig = AnnotatedSignature {
        name: "ok_func".to_string(),
        lifetimes: vec![LifetimeParam::new("'a".to_string())],
        parameters: vec![AnnotatedParameter {
            name: "x".to_string(),
            param_type: AnnotatedType::Simple(HirType::Int),
        }],
        return_type: AnnotatedType::Reference {
            inner: Box::new(AnnotatedType::Simple(HirType::Int)),
            mutable: false,
            lifetime: Some(LifetimeParam::new("'a".to_string())),
        },
    };

    assert!(annotator.validate_constraints(&sig).is_ok());
}

#[test]
fn validate_constraints_ok_when_return_lifetime_found_in_param() {
    let annotator = LifetimeAnnotator::new();

    // Return lifetime 'a is in a parameter's type
    let sig = AnnotatedSignature {
        name: "param_func".to_string(),
        lifetimes: vec![], // Not in signature lifetimes
        parameters: vec![AnnotatedParameter {
            name: "data".to_string(),
            param_type: AnnotatedType::Reference {
                inner: Box::new(AnnotatedType::Simple(HirType::Int)),
                mutable: false,
                lifetime: Some(LifetimeParam::new("'a".to_string())),
            },
        }],
        return_type: AnnotatedType::Reference {
            inner: Box::new(AnnotatedType::Simple(HirType::Int)),
            mutable: false,
            lifetime: Some(LifetimeParam::new("'a".to_string())),
        },
    };

    assert!(annotator.validate_constraints(&sig).is_ok());
}

#[test]
fn validate_constraints_ok_when_return_is_simple_type() {
    let annotator = LifetimeAnnotator::new();

    // No reference return type, so no lifetime validation needed
    let sig = AnnotatedSignature {
        name: "simple_func".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Int),
    };

    assert!(annotator.validate_constraints(&sig).is_ok());
}

#[test]
fn validate_constraints_ok_when_return_ref_has_no_lifetime() {
    let annotator = LifetimeAnnotator::new();

    // Reference return type but no lifetime annotation (lifetime: None)
    let sig = AnnotatedSignature {
        name: "no_lt_func".to_string(),
        lifetimes: vec![],
        parameters: vec![],
        return_type: AnnotatedType::Reference {
            inner: Box::new(AnnotatedType::Simple(HirType::Int)),
            mutable: false,
            lifetime: None,
        },
    };

    assert!(annotator.validate_constraints(&sig).is_ok());
}

// ============================================================================
// type_has_lifetime - Reference { lifetime: None, inner } recursive path
// ============================================================================

#[test]
fn type_has_lifetime_reference_none_recurses_into_inner() {
    let annotator = LifetimeAnnotator::new();

    // A reference with no lifetime wrapping a reference that HAS a lifetime
    let nested_type = AnnotatedType::Reference {
        inner: Box::new(AnnotatedType::Reference {
            inner: Box::new(AnnotatedType::Simple(HirType::Int)),
            mutable: false,
            lifetime: Some(LifetimeParam::new("'a".to_string())),
        }),
        mutable: false,
        lifetime: None, // This branch: recurse into inner
    };

    // Validate that the outer ref with lifetime: None recurses and finds 'a
    let sig = AnnotatedSignature {
        name: "nested_func".to_string(),
        lifetimes: vec![],
        parameters: vec![AnnotatedParameter {
            name: "data".to_string(),
            param_type: nested_type,
        }],
        return_type: AnnotatedType::Reference {
            inner: Box::new(AnnotatedType::Simple(HirType::Int)),
            mutable: false,
            lifetime: Some(LifetimeParam::new("'a".to_string())),
        },
    };

    // Should be Ok because 'a is found recursively in the parameter
    assert!(annotator.validate_constraints(&sig).is_ok());
}

#[test]
fn type_has_lifetime_reference_none_inner_simple_returns_false() {
    let annotator = LifetimeAnnotator::new();

    // A reference with no lifetime wrapping a Simple type (no lifetime found)
    let outer_type = AnnotatedType::Reference {
        inner: Box::new(AnnotatedType::Simple(HirType::Int)),
        mutable: false,
        lifetime: None,
    };

    let sig = AnnotatedSignature {
        name: "no_inner_lt".to_string(),
        lifetimes: vec![],
        parameters: vec![AnnotatedParameter {
            name: "data".to_string(),
            param_type: outer_type,
        }],
        return_type: AnnotatedType::Reference {
            inner: Box::new(AnnotatedType::Simple(HirType::Int)),
            mutable: false,
            lifetime: Some(LifetimeParam::new("'c".to_string())),
        },
    };

    // Should be Err because 'c not found anywhere
    assert!(annotator.validate_constraints(&sig).is_err());
}

// ============================================================================
// annotate_type - reference with empty lifetime_params gets None
// ============================================================================

#[test]
fn annotate_reference_type_without_lifetime_params() {
    let annotator = LifetimeAnnotator::new();

    // Function with reference param but no references detected (edge case)
    // Directly test annotate_type with empty lifetime_params
    let func = HirFunction::new(
        "no_ref_return".to_string(),
        HirType::Int,
        vec![HirParameter::new("x".to_string(), HirType::Int)],
    );

    let sig = annotator.annotate_function(&func);
    // Non-reference params get Simple annotation
    assert!(matches!(
        sig.parameters[0].param_type,
        AnnotatedType::Simple(HirType::Int)
    ));
}

// ============================================================================
// infer_lifetime_parameters - return ref with lifetimes map populated
// ============================================================================

#[test]
fn infer_lifetime_params_returns_ref_and_nonempty_lifetimes() {
    let annotator = LifetimeAnnotator::new();

    // Function that returns a reference and has a body with variables
    // This covers the `returns_ref && !lifetimes.is_empty()` branch
    let func = HirFunction::new_with_body(
        "return_ref".to_string(),
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
        vec![
            HirStatement::VariableDeclaration {
                name: "local".to_string(),
                var_type: HirType::Int,
                initializer: Some(HirExpression::IntLiteral(42)),
            },
            HirStatement::Return(Some(HirExpression::Variable("data".to_string()))),
        ],
    );

    let sig = annotator.annotate_function(&func);
    assert!(!sig.lifetimes.is_empty(), "Should have lifetime params");
    assert!(matches!(
        sig.return_type,
        AnnotatedType::Reference {
            lifetime: Some(_),
            ..
        }
    ));
}

#[test]
fn infer_lifetime_params_only_return_ref_no_ref_params() {
    let annotator = LifetimeAnnotator::new();

    // Function returns a reference but has no reference parameters
    // This covers returns_ref = true, has_ref_params = false
    let func = HirFunction::new(
        "return_only_ref".to_string(),
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
        vec![HirParameter::new("x".to_string(), HirType::Int)],
    );

    let sig = annotator.annotate_function(&func);
    assert!(
        !sig.lifetimes.is_empty(),
        "Return ref should trigger lifetime param"
    );
}

// ============================================================================
// annotate_type - mutable reference annotation
// ============================================================================

#[test]
fn annotate_mutable_reference_preserves_mutability() {
    let annotator = LifetimeAnnotator::new();

    let func = HirFunction::new(
        "mut_ref".to_string(),
        HirType::Void,
        vec![HirParameter::new(
            "data".to_string(),
            HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: true,
            },
        )],
    );

    let sig = annotator.annotate_function(&func);
    if let AnnotatedType::Reference {
        mutable, lifetime, ..
    } = &sig.parameters[0].param_type
    {
        assert!(*mutable);
        assert!(lifetime.is_some());
    } else {
        panic!("Expected reference type");
    }
}

// ============================================================================
// AnnotatedSignature field coverage
// ============================================================================

#[test]
fn annotated_signature_all_fields_accessible() {
    let sig = AnnotatedSignature {
        name: "test_func".to_string(),
        lifetimes: vec![LifetimeParam::standard(0), LifetimeParam::standard(1)],
        parameters: vec![
            AnnotatedParameter {
                name: "a".to_string(),
                param_type: AnnotatedType::Simple(HirType::Int),
            },
            AnnotatedParameter {
                name: "b".to_string(),
                param_type: AnnotatedType::Reference {
                    inner: Box::new(AnnotatedType::Simple(HirType::Float)),
                    mutable: true,
                    lifetime: Some(LifetimeParam::standard(0)),
                },
            },
        ],
        return_type: AnnotatedType::Simple(HirType::Void),
    };

    assert_eq!(sig.name, "test_func");
    assert_eq!(sig.lifetimes.len(), 2);
    assert_eq!(sig.parameters.len(), 2);
    assert_eq!(sig.parameters[0].name, "a");
    assert_eq!(sig.parameters[1].name, "b");
}

// ============================================================================
// generate_lifetime_syntax with multiple lifetimes
// ============================================================================

#[test]
fn generate_lifetime_syntax_three_lifetimes() {
    let annotator = LifetimeAnnotator::new();
    let lifetimes = vec![
        LifetimeParam::standard(0),
        LifetimeParam::standard(1),
        LifetimeParam::standard(2),
    ];
    let syntax = annotator.generate_lifetime_syntax(&lifetimes);
    assert_eq!(syntax, "<'a, 'b, 'c>");
}

#[test]
fn generate_lifetime_syntax_custom_names() {
    let annotator = LifetimeAnnotator::new();
    let lifetimes = vec![
        LifetimeParam::new("'input".to_string()),
        LifetimeParam::new("'output".to_string()),
    ];
    let syntax = annotator.generate_lifetime_syntax(&lifetimes);
    assert_eq!(syntax, "<'input, 'output>");
}

// ============================================================================
// annotate_function with various function shapes
// ============================================================================

#[test]
fn annotate_function_no_params_no_ref_return() {
    let annotator = LifetimeAnnotator::new();
    let func = HirFunction::new("void_func".to_string(), HirType::Void, vec![]);

    let sig = annotator.annotate_function(&func);
    assert_eq!(sig.name, "void_func");
    assert!(sig.lifetimes.is_empty());
    assert!(sig.parameters.is_empty());
    assert!(matches!(sig.return_type, AnnotatedType::Simple(HirType::Void)));
}

#[test]
fn annotate_function_mixed_params_ref_and_nonref() {
    let annotator = LifetimeAnnotator::new();
    let func = HirFunction::new(
        "mixed".to_string(),
        HirType::Int,
        vec![
            HirParameter::new("count".to_string(), HirType::Int),
            HirParameter::new(
                "data".to_string(),
                HirType::Reference {
                    inner: Box::new(HirType::Char),
                    mutable: false,
                },
            ),
            HirParameter::new("flag".to_string(), HirType::Int),
        ],
    );

    let sig = annotator.annotate_function(&func);
    assert!(!sig.lifetimes.is_empty());
    assert_eq!(sig.parameters.len(), 3);
    assert!(matches!(
        sig.parameters[0].param_type,
        AnnotatedType::Simple(HirType::Int)
    ));
    assert!(matches!(
        sig.parameters[1].param_type,
        AnnotatedType::Reference { .. }
    ));
    assert!(matches!(
        sig.parameters[2].param_type,
        AnnotatedType::Simple(HirType::Int)
    ));
}

// ============================================================================
// AnnotatedType Debug/Clone/PartialEq coverage
// ============================================================================

#[test]
fn annotated_type_clone_and_eq() {
    let t1 = AnnotatedType::Reference {
        inner: Box::new(AnnotatedType::Simple(HirType::Int)),
        mutable: true,
        lifetime: Some(LifetimeParam::standard(0)),
    };
    let t2 = t1.clone();
    assert_eq!(t1, t2);
}

#[test]
fn annotated_parameter_clone_and_eq() {
    let p1 = AnnotatedParameter {
        name: "x".to_string(),
        param_type: AnnotatedType::Simple(HirType::Int),
    };
    let p2 = p1.clone();
    assert_eq!(p1, p2);
}

#[test]
fn annotated_signature_clone_and_eq() {
    let sig1 = AnnotatedSignature {
        name: "f".to_string(),
        lifetimes: vec![LifetimeParam::standard(0)],
        parameters: vec![],
        return_type: AnnotatedType::Simple(HirType::Int),
    };
    let sig2 = sig1.clone();
    assert_eq!(sig1, sig2);
}

#[test]
fn lifetime_param_debug_format() {
    let lp = LifetimeParam::new("'a".to_string());
    let debug_str = format!("{:?}", lp);
    assert!(debug_str.contains("'a"));
}
