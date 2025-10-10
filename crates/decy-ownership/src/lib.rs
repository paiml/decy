//! Ownership and lifetime inference for C-to-Rust conversion.
//!
//! **CRITICAL COMPONENT**: This is the most important module for quality transpilation.
//! Infers Rust ownership patterns and lifetime annotations from C pointer usage.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

pub mod borrow_gen;
pub mod dataflow;
pub mod inference;
