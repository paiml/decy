//! Static analysis and type inference for C code.
//!
//! Provides control flow analysis, data flow analysis, and type inference.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

pub mod lock_analysis;
pub mod patterns;
pub mod tagged_union_analysis;
