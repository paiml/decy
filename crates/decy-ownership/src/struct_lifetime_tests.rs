//! Tests for struct field lifetime annotations.

use super::*;
use decy_hir::HirType;

#[test]
fn test_detect_reference_fields() {
    // Test detecting which struct fields contain references
    // For a struct like:
    // struct Data {
    //     ptr: *int,
    //     value: int
    // }
    // Should detect that 'ptr' needs lifetime annotation

    let struct_fields = vec![
        ("ptr", HirType::Pointer(Box::new(HirType::Int))),
        ("value", HirType::Int),
    ];

    let annotator = StructLifetimeAnnotator::new();
    let reference_fields = annotator.detect_reference_fields(&struct_fields);

    // Should detect that 'ptr' needs a lifetime
    assert!(
        reference_fields.contains(&"ptr".to_string()),
        "Should detect pointer field as needing lifetime"
    );
    assert!(
        !reference_fields.contains(&"value".to_string()),
        "Should not mark plain int as needing lifetime"
    );
}

#[test]
fn test_infer_struct_lifetimes() {
    // Test inferring lifetime parameters needed for a struct
    // Struct with one reference field needs one lifetime
    let struct_fields = vec![(
        "data",
        HirType::Reference {
            inner: Box::new(HirType::Int),
            mutable: false,
        },
    )];

    let annotator = StructLifetimeAnnotator::new();
    let lifetimes = annotator.infer_struct_lifetimes("Data", &struct_fields);

    assert_eq!(lifetimes.len(), 1, "Should infer one lifetime parameter");
    assert_eq!(lifetimes[0].name, "'a", "Should use 'a for first lifetime");
}

#[test]
fn test_generate_struct_lifetime_syntax() {
    // Test generating the <'a, 'b> syntax for struct declarations
    let lifetimes = vec![
        LifetimeParam::standard(0), // 'a
    ];

    let annotator = StructLifetimeAnnotator::new();
    let syntax = annotator.generate_struct_lifetime_syntax(&lifetimes);

    assert_eq!(syntax, "<'a>", "Should generate correct lifetime syntax");
}

#[test]
fn test_annotate_fields() {
    // Test annotating struct fields with lifetime parameters
    let struct_fields = vec![
        (
            "ptr",
            HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: false,
            },
        ),
        ("value", HirType::Int),
    ];

    let lifetimes = vec![LifetimeParam::standard(0)];

    let annotator = StructLifetimeAnnotator::new();
    let annotated_fields = annotator.annotate_fields(&struct_fields, &lifetimes);

    assert_eq!(annotated_fields.len(), 2, "Should have two fields");

    // First field should have lifetime annotation
    assert_eq!(annotated_fields[0].name, "ptr");
    match &annotated_fields[0].field_type {
        AnnotatedType::Reference { lifetime, .. } => {
            assert!(lifetime.is_some(), "Reference field should have lifetime");
            assert_eq!(lifetime.as_ref().unwrap().name, "'a");
        }
        _ => panic!("Expected reference type for ptr field"),
    }

    // Second field should not have lifetime
    assert_eq!(annotated_fields[1].name, "value");
    assert!(
        matches!(&annotated_fields[1].field_type, AnnotatedType::Simple(_)),
        "Plain int should be Simple type"
    );
}

#[test]
fn test_struct_with_no_references() {
    // Test that struct with no reference fields gets no lifetime parameters
    let struct_fields = vec![("x", HirType::Int), ("y", HirType::Int)];

    let annotator = StructLifetimeAnnotator::new();
    let lifetimes = annotator.infer_struct_lifetimes("Point", &struct_fields);

    assert_eq!(
        lifetimes.len(),
        0,
        "Struct with no references should have no lifetimes"
    );
}

#[test]
fn test_struct_with_multiple_reference_fields() {
    // Test struct with multiple reference fields
    // All should share the same lifetime for simplicity
    let struct_fields = vec![
        (
            "first",
            HirType::Reference {
                inner: Box::new(HirType::Int),
                mutable: false,
            },
        ),
        (
            "second",
            HirType::Reference {
                inner: Box::new(HirType::Char),
                mutable: true,
            },
        ),
    ];

    let annotator = StructLifetimeAnnotator::new();
    let lifetimes = annotator.infer_struct_lifetimes("MultiRef", &struct_fields);

    // For now, use single lifetime for all references
    assert_eq!(lifetimes.len(), 1, "Should use one lifetime for simplicity");
}

#[test]
fn test_annotate_struct_declaration() {
    // Test end-to-end struct declaration annotation
    let struct_fields = vec![
        (
            "name",
            HirType::Reference {
                inner: Box::new(HirType::Char),
                mutable: false,
            },
        ),
        ("age", HirType::Int),
    ];

    let annotator = StructLifetimeAnnotator::new();
    let annotated = annotator.annotate_struct("Person", &struct_fields);

    assert_eq!(annotated.name, "Person");
    assert_eq!(annotated.lifetimes.len(), 1);
    assert_eq!(annotated.fields.len(), 2);

    // Generate the full syntax
    let syntax = format!(
        "struct {}{} {{",
        annotated.name,
        annotator.generate_struct_lifetime_syntax(&annotated.lifetimes)
    );

    assert_eq!(syntax, "struct Person<'a> {");
}

#[test]
fn test_nested_pointer_in_struct() {
    // Test struct field with nested pointer types
    let struct_fields = vec![(
        "ptr_ptr",
        HirType::Pointer(Box::new(HirType::Pointer(Box::new(HirType::Int)))),
    )];

    let annotator = StructLifetimeAnnotator::new();
    let reference_fields = annotator.detect_reference_fields(&struct_fields);

    // Nested pointers should also be detected
    assert!(
        reference_fields.contains(&"ptr_ptr".to_string()),
        "Should detect nested pointers"
    );
}

#[test]
fn test_pointer_converts_to_reference_with_lifetime() {
    // Mutation testing found: deleting Pointer match arm doesn't fail tests
    // Test that Pointer types are converted to References with lifetimes
    let struct_fields = vec![("ptr", HirType::Pointer(Box::new(HirType::Int)))];

    let lifetimes = vec![LifetimeParam::standard(0)];

    let annotator = StructLifetimeAnnotator::new();
    let annotated_fields = annotator.annotate_fields(&struct_fields, &lifetimes);

    assert_eq!(annotated_fields.len(), 1);
    assert_eq!(annotated_fields[0].name, "ptr");

    // Critical: Pointer should become Reference, not Simple
    match &annotated_fields[0].field_type {
        AnnotatedType::Reference {
            lifetime, mutable, ..
        } => {
            assert!(
                lifetime.is_some(),
                "Pointer should be converted to Reference with lifetime"
            );
            assert_eq!(lifetime.as_ref().unwrap().name, "'a");
            assert!(
                !(*mutable),
                "Default pointer conversion should be immutable"
            );
        }
        AnnotatedType::Simple(_) => {
            panic!("Pointer should NOT be Simple type - should be converted to Reference")
        }
    }
}

#[test]
fn test_pointer_without_lifetimes_gets_none() {
    // Mutation testing found: deleting ! in !lifetimes.is_empty() doesn't fail
    // Test that when no lifetimes are provided, pointer gets None lifetime
    let struct_fields = vec![("ptr", HirType::Pointer(Box::new(HirType::Int)))];

    let empty_lifetimes: Vec<LifetimeParam> = vec![];

    let annotator = StructLifetimeAnnotator::new();
    let annotated_fields = annotator.annotate_fields(&struct_fields, &empty_lifetimes);

    assert_eq!(annotated_fields.len(), 1);

    match &annotated_fields[0].field_type {
        AnnotatedType::Reference { lifetime, .. } => {
            assert!(
                lifetime.is_none(),
                "When no lifetimes provided, should get None lifetime"
            );
        }
        _ => panic!("Expected Reference type for pointer field"),
    }
}

#[test]
fn test_pointer_with_lifetimes_gets_lifetime() {
    // Test that when lifetimes ARE provided, pointer gets the lifetime
    let struct_fields = vec![("ptr", HirType::Pointer(Box::new(HirType::Int)))];

    let lifetimes = vec![LifetimeParam::standard(0)];

    let annotator = StructLifetimeAnnotator::new();
    let annotated_fields = annotator.annotate_fields(&struct_fields, &lifetimes);

    match &annotated_fields[0].field_type {
        AnnotatedType::Reference { lifetime, .. } => {
            assert!(
                lifetime.is_some(),
                "When lifetimes provided, should get Some(lifetime)"
            );
            assert_eq!(lifetime.as_ref().unwrap().name, "'a");
        }
        _ => panic!("Expected Reference type for pointer field"),
    }
}
