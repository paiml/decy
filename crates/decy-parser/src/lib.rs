//! C AST parsing using clang-sys.
//!
//! This crate provides a production-grade C parser using LLVM/Clang bindings.

#![warn(missing_docs)]
#![warn(clippy::all)]
// Note: clang-sys requires unsafe for FFI, but we allow it only in this crate
#![allow(unsafe_code)]

pub mod parser;

pub use parser::{Ast, CParser, Function, Parameter, Type};
