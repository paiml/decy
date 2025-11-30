//! Static analysis and type inference for C code.
//!
//! Provides control flow analysis, data flow analysis, and type inference.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

pub mod lock_analysis;
pub mod output_params;
pub mod patterns;
pub mod subprocess_analysis;
pub mod tagged_union_analysis;
pub mod void_ptr_analysis;
