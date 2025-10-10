//! Borrow code generation from ownership inference.
//!
//! This module generates Rust borrow syntax (&T, &mut T) from ownership
//! inference results, transforming C pointers into safe Rust references.

use crate::inference::{OwnershipInference, OwnershipKind};
use decy_hir::{HirFunction, HirParameter, HirType};
use std::collections::HashMap;

/// Borrow code generator.
///
/// Transforms HIR based on ownership inference to generate
/// idiomatic Rust borrow code.
#[derive(Debug)]
pub struct BorrowGenerator;

impl BorrowGenerator {
    /// Create a new borrow generator.
    pub fn new() -> Self {
        Self
    }

    /// Transform a HIR type based on ownership inference.
    ///
    /// Converts C pointer types to appropriate Rust borrow types:
    /// - Owning pointers → `Box<T>` (already handled by Box transformer)
    /// - ImmutableBorrow → &T
    /// - MutableBorrow → &mut T
    /// - Unknown → *mut T (falls back to raw pointer)
    pub fn transform_type(
        &self,
        hir_type: &HirType,
        variable_name: &str,
        inferences: &HashMap<String, OwnershipInference>,
    ) -> HirType {
        match hir_type {
            HirType::Pointer(inner) => {
                // Look up ownership inference for this variable
                if let Some(inference) = inferences.get(variable_name) {
                    match inference.kind {
                        OwnershipKind::Owning => {
                            // Owning pointers become Box<T>
                            HirType::Box(inner.clone())
                        }
                        OwnershipKind::ImmutableBorrow => {
                            // Immutable borrows become &T
                            HirType::Reference {
                                inner: inner.clone(),
                                mutable: false,
                            }
                        }
                        OwnershipKind::MutableBorrow => {
                            // Mutable borrows become &mut T
                            HirType::Reference {
                                inner: inner.clone(),
                                mutable: true,
                            }
                        }
                        OwnershipKind::Unknown => {
                            // Unknown cases fall back to raw pointer
                            HirType::Pointer(inner.clone())
                        }
                    }
                } else {
                    // No inference available, keep as pointer
                    HirType::Pointer(inner.clone())
                }
            }
            // Non-pointer types remain unchanged
            _ => hir_type.clone(),
        }
    }

    /// Transform function parameters based on ownership inference.
    ///
    /// Converts pointer parameters to appropriate borrow types.
    pub fn transform_parameters(
        &self,
        params: &[HirParameter],
        inferences: &HashMap<String, OwnershipInference>,
    ) -> Vec<HirParameter> {
        params
            .iter()
            .map(|param| {
                let transformed_type =
                    self.transform_type(param.param_type(), param.name(), inferences);
                HirParameter::new(param.name().to_string(), transformed_type)
            })
            .collect()
    }

    /// Transform a function signature based on ownership inference.
    ///
    /// Returns a new HirFunction with transformed parameter types.
    pub fn transform_function(
        &self,
        func: &HirFunction,
        inferences: &HashMap<String, OwnershipInference>,
    ) -> HirFunction {
        let transformed_params = self.transform_parameters(func.parameters(), inferences);

        HirFunction::new_with_body(
            func.name().to_string(),
            func.return_type().clone(),
            transformed_params,
            func.body().to_vec(),
        )
    }
}

impl Default for BorrowGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "borrow_gen_tests.rs"]
mod borrow_gen_tests;
