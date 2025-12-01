//! Ownership and lifetime inference for C-to-Rust conversion.
//!
//! **CRITICAL COMPONENT**: This is the most important module for quality transpilation.
//! Infers Rust ownership patterns and lifetime annotations from C pointer usage.
//!
//! # ML-Enhanced Features (DECY-ML-001, DECY-ML-003)
//!
//! This crate includes ML-enhanced ownership inference features:
//! - [`OwnershipFeatures`]: 142-dimension feature vector for batch ML processing
//! - [`OwnershipDefect`]: 8-category defect taxonomy (DECY-O-001 through DECY-O-008)
//! - [`InferredOwnership`]: Predicted Rust ownership kinds
//! - [`OwnershipPrediction`]: Ownership with confidence score and fallback

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

pub mod array_slice;
pub mod borrow_gen;
pub mod dataflow;
pub mod inference;
pub mod lifetime;
pub mod lifetime_gen;
pub mod ml_features;
pub mod struct_lifetime;

// Re-export ML feature types at crate root for convenience
pub use ml_features::{
    AllocationKind, ErrorPattern, ErrorSeverity, FeatureExtractor, InferredOwnership,
    OwnershipDefect, OwnershipErrorKind, OwnershipFeatures, OwnershipFeaturesBuilder,
    OwnershipPrediction, PatternLibrary, Severity, SuggestedFix,
};

#[cfg(test)]
mod ml_features_tests;
