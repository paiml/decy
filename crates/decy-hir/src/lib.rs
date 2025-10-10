//! High-level Intermediate Representation for C-to-Rust transpilation.
//!
//! The HIR is a Rust-oriented representation that bridges C AST and Rust code generation.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

#[cfg(test)]
#[path = "hir_tests.rs"]
mod hir_tests;
