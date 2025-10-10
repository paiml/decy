//! Rust code generation from HIR with minimal unsafe blocks.
//!
//! Generates idiomatic Rust code with <5 unsafe blocks per 1000 LOC.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

#[cfg(test)]
#[path = "codegen_tests.rs"]
mod codegen_tests;
