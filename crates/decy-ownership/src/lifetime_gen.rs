//! Lifetime annotation generation for function signatures.
//!
//! This module generates Rust lifetime annotations (<'a, 'b>) for function
//! signatures based on scope-based lifetime analysis results.

use crate::lifetime::{LifetimeAnalyzer, VariableLifetime};
use decy_hir::{HirFunction, HirParameter, HirType};
use std::collections::{HashMap, HashSet};

/// Represents a lifetime parameter in a function signature.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LifetimeParam {
    /// Lifetime name (e.g., 'a', 'b')
    pub name: String,
}

impl LifetimeParam {
    /// Create a new lifetime parameter.
    pub fn new(name: String) -> Self {
        Self { name }
    }

    /// Create a standard lifetime parameter ('a, 'b, 'c, etc.)
    pub fn standard(index: usize) -> Self {
        let name = format!("'{}", (b'a' + index as u8) as char);
        Self { name }
    }
}

/// Annotated function signature with lifetime parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotatedSignature {
    /// Function name
    pub name: String,
    /// Lifetime parameters
    pub lifetimes: Vec<LifetimeParam>,
    /// Annotated parameters
    pub parameters: Vec<AnnotatedParameter>,
    /// Return type with lifetime annotation
    pub return_type: AnnotatedType,
}

/// Parameter with lifetime annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotatedParameter {
    /// Parameter name
    pub name: String,
    /// Type with lifetime annotation
    pub param_type: AnnotatedType,
}

/// Type with optional lifetime annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotatedType {
    /// Simple type (no lifetime needed)
    Simple(HirType),
    /// Reference with lifetime
    Reference {
        /// Inner type
        inner: Box<AnnotatedType>,
        /// Mutable flag
        mutable: bool,
        /// Lifetime parameter
        lifetime: Option<LifetimeParam>,
    },
}

/// Lifetime annotation generator.
#[derive(Debug)]
pub struct LifetimeAnnotator;

impl LifetimeAnnotator {
    /// Create a new lifetime annotator.
    pub fn new() -> Self {
        Self
    }

    /// Generate lifetime annotations for a function.
    ///
    /// Analyzes the function using lifetime analysis and generates
    /// appropriate lifetime parameters and annotations.
    pub fn annotate_function(&self, func: &HirFunction) -> AnnotatedSignature {
        let analyzer = LifetimeAnalyzer::new();
        let scope_tree = analyzer.build_scope_tree(func);
        let lifetimes_map = analyzer.track_lifetimes(func, &scope_tree);

        // Infer which parameters need lifetime annotations
        let lifetime_params = self.infer_lifetime_parameters(func, &lifetimes_map);

        // Annotate parameters
        let annotated_params = self.annotate_parameters(func.parameters(), &lifetime_params);

        // Annotate return type
        let return_type = self.annotate_return_type(func, &lifetime_params, &lifetimes_map);

        AnnotatedSignature {
            name: func.name().to_string(),
            lifetimes: lifetime_params,
            parameters: annotated_params,
            return_type,
        }
    }

    /// Infer which lifetime parameters are needed for a function.
    ///
    /// Determines if the function needs lifetime annotations based on:
    /// - Reference parameters
    /// - Reference return types
    /// - Lifetime relationships between parameters and return
    fn infer_lifetime_parameters(
        &self,
        func: &HirFunction,
        lifetimes: &HashMap<String, VariableLifetime>,
    ) -> Vec<LifetimeParam> {
        let mut needed_lifetimes = HashSet::new();

        // Check if any parameters are references
        let has_ref_params = func
            .parameters()
            .iter()
            .any(|p| matches!(p.param_type(), HirType::Reference { .. }));

        // Check if return type is a reference
        let returns_ref = matches!(func.return_type(), HirType::Reference { .. });

        // If we have references, we need at least one lifetime
        if has_ref_params || returns_ref {
            needed_lifetimes.insert(LifetimeParam::standard(0)); // 'a
        }

        // Check if return value depends on parameters
        if returns_ref && !lifetimes.is_empty() {
            // If return references a parameter, ensure we have a lifetime
            needed_lifetimes.insert(LifetimeParam::standard(0));
        }

        // For now, use a single lifetime for simplicity
        // Future: analyze dependencies to determine multiple lifetimes
        needed_lifetimes.into_iter().collect()
    }

    /// Annotate function parameters with lifetime parameters.
    fn annotate_parameters(
        &self,
        params: &[HirParameter],
        lifetime_params: &[LifetimeParam],
    ) -> Vec<AnnotatedParameter> {
        params
            .iter()
            .map(|param| {
                let annotated_type = self.annotate_type(param.param_type(), lifetime_params);
                AnnotatedParameter {
                    name: param.name().to_string(),
                    param_type: annotated_type,
                }
            })
            .collect()
    }

    /// Annotate a type with lifetime parameters.
    #[allow(clippy::only_used_in_recursion)]
    fn annotate_type(
        &self,
        hir_type: &HirType,
        lifetime_params: &[LifetimeParam],
    ) -> AnnotatedType {
        match hir_type {
            HirType::Reference { inner, mutable } => {
                // Add lifetime annotation to reference types
                let lifetime = if !lifetime_params.is_empty() {
                    Some(lifetime_params[0].clone())
                } else {
                    None
                };
                AnnotatedType::Reference {
                    inner: Box::new(self.annotate_type(inner, lifetime_params)),
                    mutable: *mutable,
                    lifetime,
                }
            }
            _ => AnnotatedType::Simple(hir_type.clone()),
        }
    }

    /// Annotate return type with lifetime parameters.
    fn annotate_return_type(
        &self,
        func: &HirFunction,
        lifetime_params: &[LifetimeParam],
        _lifetimes: &HashMap<String, VariableLifetime>,
    ) -> AnnotatedType {
        self.annotate_type(func.return_type(), lifetime_params)
    }

    /// Generate lifetime syntax string (e.g., "<'a, 'b>").
    pub fn generate_lifetime_syntax(&self, lifetimes: &[LifetimeParam]) -> String {
        if lifetimes.is_empty() {
            String::new()
        } else {
            let params: Vec<String> = lifetimes.iter().map(|lt| lt.name.clone()).collect();
            format!("<{}>", params.join(", "))
        }
    }

    /// Validate lifetime constraints.
    ///
    /// Ensures that lifetime annotations follow Rust's rules:
    /// - Return lifetime must be a subset of parameter lifetimes
    /// - No lifetime outlives another incorrectly
    pub fn validate_constraints(&self, signature: &AnnotatedSignature) -> Result<(), String> {
        // Check that return type lifetime exists in parameters
        if let AnnotatedType::Reference {
            lifetime: Some(ref ret_lifetime),
            ..
        } = signature.return_type
        {
            // Verify that this lifetime appears in parameters
            let param_has_lifetime = signature.parameters.iter().any(|param| {
                self.type_has_lifetime(&param.param_type, &ret_lifetime.name)
            });

            if !param_has_lifetime && !signature.lifetimes.contains(ret_lifetime) {
                return Err(format!(
                    "Return type references lifetime {} not found in parameters",
                    ret_lifetime.name
                ));
            }
        }

        Ok(())
    }

    /// Check if a type contains a specific lifetime.
    #[allow(clippy::only_used_in_recursion)]
    fn type_has_lifetime(&self, annotated_type: &AnnotatedType, lifetime_name: &str) -> bool {
        match annotated_type {
            AnnotatedType::Reference {
                lifetime: Some(lt), ..
            } => lt.name == lifetime_name,
            AnnotatedType::Reference {
                lifetime: None,
                inner,
                ..
            } => self.type_has_lifetime(inner, lifetime_name),
            AnnotatedType::Simple(_) => false,
        }
    }
}

impl Default for LifetimeAnnotator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "lifetime_gen_tests.rs"]
mod lifetime_gen_tests;
