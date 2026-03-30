//! C AST parsing using clang-sys.
//!
//! This crate provides a production-grade C parser using LLVM/Clang bindings.

#![warn(missing_docs)]
#![warn(clippy::all)]
// Note: clang-sys requires unsafe for FFI, but we allow it only in this crate
#![allow(unsafe_code)]

pub mod ast_types;
pub mod diagnostic;
pub(crate) mod extract;
pub mod parser;

pub use diagnostic::{Diagnostic, DiagnosticError, ErrorCategory, Severity};
pub use parser::{
    Ast, CParser, Class, CudaQualifier, Expression, Function, Namespace, Parameter, Statement,
    Struct, StructField, Type, Variable,
};
