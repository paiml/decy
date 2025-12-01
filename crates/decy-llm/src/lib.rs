//! LLM context builder for C-to-Rust transpilation.
//!
//! Formats static analysis results as structured JSON context for LLM prompts.
//! This enables LLM-guided code generation with ownership, lifetime, and
//! concurrency analysis information.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

pub mod context_builder;
pub mod llm_codegen;
pub mod verifier;

pub use context_builder::{AnalysisContext, ContextBuilder, FunctionContext};
pub use llm_codegen::{CodegenPrompt, GeneratedCode, LlmCodegen, LlmError};
pub use verifier::{CodeVerifier, IterationContext, VerificationLoop, VerificationResult};

#[cfg(test)]
mod verifier_tests;
