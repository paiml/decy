//! Struct field lifetime annotation generation.
//!
//! This module generates Rust lifetime annotations for struct fields
//! with references, transforming C structs with pointers into safe
//! Rust structs with lifetime-annotated references.

use crate::lifetime_gen::{AnnotatedType, LifetimeParam};
use decy_hir::HirType;

/// Annotated struct field with lifetime information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotatedField {
    /// Field name
    pub name: String,
    /// Field type with lifetime annotation
    pub field_type: AnnotatedType,
}

/// Annotated struct declaration with lifetime parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotatedStruct {
    /// Struct name
    pub name: String,
    /// Lifetime parameters
    pub lifetimes: Vec<LifetimeParam>,
    /// Annotated fields
    pub fields: Vec<AnnotatedField>,
}

/// Struct lifetime annotation generator.
#[derive(Debug)]
pub struct StructLifetimeAnnotator;

impl StructLifetimeAnnotator {
    /// Create a new struct lifetime annotator.
    pub fn new() -> Self {
        Self
    }

    /// Detect which struct fields contain references that need lifetime annotations.
    ///
    /// Returns the names of fields that contain pointers or references.
    pub fn detect_reference_fields(&self, fields: &[(&str, HirType)]) -> Vec<String> {
        fields
            .iter()
            .filter_map(|(name, field_type)| {
                if self.type_needs_lifetime(field_type) {
                    Some(name.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if a type needs a lifetime annotation.
    fn type_needs_lifetime(&self, hir_type: &HirType) -> bool {
        matches!(hir_type, HirType::Pointer(_) | HirType::Reference { .. })
    }

    /// Infer lifetime parameters needed for a struct.
    ///
    /// Currently uses a single lifetime parameter for all reference fields.
    /// Future enhancement: analyze field relationships for multiple lifetimes.
    pub fn infer_struct_lifetimes(
        &self,
        _struct_name: &str,
        fields: &[(&str, HirType)],
    ) -> Vec<LifetimeParam> {
        // Check if any fields need lifetimes
        let has_references = fields
            .iter()
            .any(|(_, field_type)| self.type_needs_lifetime(field_type));

        if has_references {
            // Use a single lifetime for all references
            vec![LifetimeParam::standard(0)] // 'a
        } else {
            vec![]
        }
    }

    /// Generate struct lifetime syntax (e.g., "<'a>").
    pub fn generate_struct_lifetime_syntax(&self, lifetimes: &[LifetimeParam]) -> String {
        if lifetimes.is_empty() {
            String::new()
        } else {
            let params: Vec<String> = lifetimes.iter().map(|lt| lt.name.clone()).collect();
            format!("<{}>", params.join(", "))
        }
    }

    /// Annotate struct fields with lifetime parameters.
    pub fn annotate_fields(
        &self,
        fields: &[(&str, HirType)],
        lifetimes: &[LifetimeParam],
    ) -> Vec<AnnotatedField> {
        fields
            .iter()
            .map(|(name, field_type)| {
                let annotated_type = self.annotate_field_type(field_type, lifetimes);
                AnnotatedField {
                    name: name.to_string(),
                    field_type: annotated_type,
                }
            })
            .collect()
    }

    /// Annotate a field type with lifetime parameters.
    #[allow(clippy::only_used_in_recursion)]
    fn annotate_field_type(
        &self,
        hir_type: &HirType,
        lifetimes: &[LifetimeParam],
    ) -> AnnotatedType {
        match hir_type {
            HirType::Reference { inner, mutable } => {
                // Add lifetime annotation to reference types
                let lifetime = if !lifetimes.is_empty() {
                    Some(lifetimes[0].clone())
                } else {
                    None
                };
                AnnotatedType::Reference {
                    inner: Box::new(self.annotate_field_type(inner, lifetimes)),
                    mutable: *mutable,
                    lifetime,
                }
            }
            HirType::Pointer(inner) => {
                // Pointers become references with lifetimes
                let lifetime = if !lifetimes.is_empty() {
                    Some(lifetimes[0].clone())
                } else {
                    None
                };
                AnnotatedType::Reference {
                    inner: Box::new(self.annotate_field_type(inner, lifetimes)),
                    mutable: false, // Default to immutable
                    lifetime,
                }
            }
            _ => AnnotatedType::Simple(hir_type.clone()),
        }
    }

    /// Annotate a complete struct declaration.
    pub fn annotate_struct(
        &self,
        struct_name: &str,
        fields: &[(&str, HirType)],
    ) -> AnnotatedStruct {
        let lifetimes = self.infer_struct_lifetimes(struct_name, fields);
        let annotated_fields = self.annotate_fields(fields, &lifetimes);

        AnnotatedStruct {
            name: struct_name.to_string(),
            lifetimes,
            fields: annotated_fields,
        }
    }
}

impl Default for StructLifetimeAnnotator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "struct_lifetime_tests.rs"]
mod struct_lifetime_tests;
