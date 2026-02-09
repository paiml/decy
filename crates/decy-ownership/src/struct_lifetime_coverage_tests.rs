//! Additional coverage tests for struct_lifetime.rs.
//!
//! Targets the remaining ~5 uncovered lines including:
//! - annotate_field_type with Reference { mutable: true } (mutable reference)
//! - annotate_field_type with non-pointer/non-reference types (Simple branch)
//! - generate_struct_lifetime_syntax with multiple lifetimes
//! - annotate_fields with empty lifetimes vec
//! - annotate_struct with only non-reference fields (no lifetimes)
//! - detect_reference_fields with empty fields vec
//! - Nested Reference types

use crate::lifetime_gen::{AnnotatedType, LifetimeParam};
use crate::struct_lifetime::StructLifetimeAnnotator;
use decy_hir::HirType;

// ============================================================================
// annotate_field_type: mutable Reference
// ============================================================================

#[test]
fn test_annotate_mutable_reference_field() {
    let annotator = StructLifetimeAnnotator::new();
    let fields = vec![(
        "data",
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: true,
        },
    )];
    let lifetimes = vec![LifetimeParam::standard(0)];
    let annotated = annotator.annotate_fields(&fields, &lifetimes);

    assert_eq!(annotated.len(), 1);
    match &annotated[0].field_type {
        AnnotatedType::Reference {
            mutable, lifetime, ..
        } => {
            assert!(*mutable, "Should preserve mutable flag");
            assert!(lifetime.is_some());
            assert_eq!(lifetime.as_ref().unwrap().name, "'a");
        }
        other => panic!("Expected Reference, got {:?}", other),
    }
}

// ============================================================================
// annotate_field_type: Reference with empty lifetimes (None lifetime)
// ============================================================================

#[test]
fn test_annotate_reference_no_lifetimes_gives_none() {
    let annotator = StructLifetimeAnnotator::new();
    let fields = vec![(
        "data",
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
    )];
    let empty_lifetimes: Vec<LifetimeParam> = vec![];
    let annotated = annotator.annotate_fields(&fields, &empty_lifetimes);

    match &annotated[0].field_type {
        AnnotatedType::Reference { lifetime, .. } => {
            assert!(
                lifetime.is_none(),
                "Reference with no lifetimes should have None lifetime"
            );
        }
        other => panic!("Expected Reference, got {:?}", other),
    }
}

// ============================================================================
// annotate_field_type: Simple type (non-pointer, non-reference)
// ============================================================================

#[test]
fn test_annotate_simple_type_float() {
    let annotator = StructLifetimeAnnotator::new();
    let fields = vec![("value", HirType::Float)];
    let lifetimes = vec![LifetimeParam::standard(0)];
    let annotated = annotator.annotate_fields(&fields, &lifetimes);

    assert_eq!(annotated.len(), 1);
    assert!(
        matches!(&annotated[0].field_type, AnnotatedType::Simple(HirType::Float)),
        "Float should be Simple type"
    );
}

#[test]
fn test_annotate_simple_type_char() {
    let annotator = StructLifetimeAnnotator::new();
    let fields = vec![("ch", HirType::Char)];
    let lifetimes = vec![];
    let annotated = annotator.annotate_fields(&fields, &lifetimes);

    assert!(matches!(
        &annotated[0].field_type,
        AnnotatedType::Simple(HirType::Char)
    ));
}

#[test]
fn test_annotate_simple_type_double() {
    let annotator = StructLifetimeAnnotator::new();
    let fields = vec![("d", HirType::Double)];
    let lifetimes = vec![];
    let annotated = annotator.annotate_fields(&fields, &lifetimes);

    assert!(matches!(
        &annotated[0].field_type,
        AnnotatedType::Simple(HirType::Double)
    ));
}

// ============================================================================
// generate_struct_lifetime_syntax: multiple lifetimes
// ============================================================================

#[test]
fn test_generate_syntax_multiple_lifetimes() {
    let annotator = StructLifetimeAnnotator::new();
    let lifetimes = vec![
        LifetimeParam::standard(0), // 'a
        LifetimeParam::standard(1), // 'b
    ];
    let syntax = annotator.generate_struct_lifetime_syntax(&lifetimes);
    assert_eq!(syntax, "<'a, 'b>");
}

#[test]
fn test_generate_syntax_three_lifetimes() {
    let annotator = StructLifetimeAnnotator::new();
    let lifetimes = vec![
        LifetimeParam::standard(0), // 'a
        LifetimeParam::standard(1), // 'b
        LifetimeParam::standard(2), // 'c
    ];
    let syntax = annotator.generate_struct_lifetime_syntax(&lifetimes);
    assert_eq!(syntax, "<'a, 'b, 'c>");
}

#[test]
fn test_generate_syntax_empty_lifetimes() {
    let annotator = StructLifetimeAnnotator::new();
    let lifetimes: Vec<LifetimeParam> = vec![];
    let syntax = annotator.generate_struct_lifetime_syntax(&lifetimes);
    assert_eq!(syntax, "");
}

// ============================================================================
// annotate_struct: struct with only non-reference fields
// ============================================================================

#[test]
fn test_annotate_struct_no_references() {
    let annotator = StructLifetimeAnnotator::new();
    let fields = vec![
        ("x", HirType::Int),
        ("y", HirType::Float),
        ("z", HirType::Double),
    ];
    let annotated = annotator.annotate_struct("Vec3", &fields);
    assert_eq!(annotated.name, "Vec3");
    assert!(annotated.lifetimes.is_empty(), "No lifetimes for plain types");
    assert_eq!(annotated.fields.len(), 3);
    for field in &annotated.fields {
        assert!(
            matches!(&field.field_type, AnnotatedType::Simple(_)),
            "All fields should be Simple"
        );
    }
}

// ============================================================================
// detect_reference_fields: empty fields
// ============================================================================

#[test]
fn test_detect_reference_fields_empty() {
    let annotator = StructLifetimeAnnotator::new();
    let fields: Vec<(&str, HirType)> = vec![];
    let refs = annotator.detect_reference_fields(&fields);
    assert!(refs.is_empty());
}

// ============================================================================
// detect_reference_fields: mixed types
// ============================================================================

#[test]
fn test_detect_reference_fields_mixed() {
    let annotator = StructLifetimeAnnotator::new();
    let fields = vec![
        ("a", HirType::Int),
        (
            "b",
            HirType::Pointer(Box::new(HirType::Char)),
        ),
        ("c", HirType::Float),
        (
            "d",
            HirType::Reference {
                inner: Box::new(HirType::Double),
                mutable: true,
            },
        ),
    ];
    let refs = annotator.detect_reference_fields(&fields);
    assert_eq!(refs.len(), 2);
    assert!(refs.contains(&"b".to_string()));
    assert!(refs.contains(&"d".to_string()));
    assert!(!refs.contains(&"a".to_string()));
    assert!(!refs.contains(&"c".to_string()));
}

// ============================================================================
// Nested pointer type annotation
// ============================================================================

#[test]
fn test_annotate_nested_pointer() {
    let annotator = StructLifetimeAnnotator::new();
    let fields = vec![(
        "nested",
        HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Int)))),
    )];
    let lifetimes = vec![LifetimeParam::standard(0)];
    let annotated = annotator.annotate_fields(&fields, &lifetimes);

    // Outer pointer becomes reference
    match &annotated[0].field_type {
        AnnotatedType::Reference { inner, lifetime, .. } => {
            assert!(lifetime.is_some());
            // Inner pointer also becomes reference
            match &**inner {
                AnnotatedType::Reference { lifetime: inner_lt, .. } => {
                    assert!(inner_lt.is_some());
                }
                other => panic!("Expected inner Reference, got {:?}", other),
            }
        }
        other => panic!("Expected outer Reference, got {:?}", other),
    }
}

// ============================================================================
// Default trait
// ============================================================================

#[test]
fn test_struct_lifetime_annotator_default() {
    let annotator: StructLifetimeAnnotator = Default::default();
    let debug = format!("{:?}", annotator);
    assert!(debug.contains("StructLifetimeAnnotator"));
}

// ============================================================================
// infer_struct_lifetimes with only pointers (no Reference types)
// ============================================================================

#[test]
fn test_infer_lifetimes_pointer_fields() {
    let annotator = StructLifetimeAnnotator::new();
    let fields = vec![
        ("ptr1", HirType::Pointer(Box::new(HirType::Int))),
        ("ptr2", HirType::Pointer(Box::new(HirType::Char))),
    ];
    let lifetimes = annotator.infer_struct_lifetimes("MultiPtr", &fields);
    assert_eq!(lifetimes.len(), 1, "Multiple pointer fields should share one lifetime");
}

// ============================================================================
// Full annotate_struct with both pointer and reference fields
// ============================================================================

#[test]
fn test_annotate_struct_mixed_ptr_and_ref() {
    let annotator = StructLifetimeAnnotator::new();
    let fields = vec![
        ("ptr_field", HirType::Pointer(Box::new(HirType::Int))),
        (
            "ref_field",
            HirType::Reference {
                inner: Box::new(HirType::Char),
                mutable: false,
            },
        ),
        ("int_field", HirType::Int),
    ];
    let annotated = annotator.annotate_struct("Mixed", &fields);

    assert_eq!(annotated.name, "Mixed");
    assert_eq!(annotated.lifetimes.len(), 1);
    assert_eq!(annotated.fields.len(), 3);

    // ptr_field -> Reference with lifetime
    assert!(matches!(&annotated.fields[0].field_type, AnnotatedType::Reference { .. }));
    // ref_field -> Reference with lifetime
    assert!(matches!(&annotated.fields[1].field_type, AnnotatedType::Reference { .. }));
    // int_field -> Simple
    assert!(matches!(&annotated.fields[2].field_type, AnnotatedType::Simple(_)));
}
