//! CITL Oracle Integration for Decy
//!
//! This crate provides integration with entrenar's Compiler-in-the-Loop Training (CITL)
//! oracle for the decy C→Rust transpiler.
//!
//! # Overview
//!
//! The oracle queries accumulated fix patterns from `.apr` files to suggest corrections
//! for rustc errors, reducing LLM dependency and achieving cost-free steady-state operation.
//!
//! # Features
//!
//! - `citl`: Enable full entrenar CITL integration (requires entrenar crate)
//!
//! # Example
//!
//! ```no_run
//! use decy_oracle::{DecyOracle, OracleConfig};
//!
//! let config = OracleConfig::default();
//! let mut oracle = DecyOracle::new(config).unwrap();
//!
//! // Query for fix suggestion
//! // let suggestion = oracle.suggest_fix(&error, &context);
//! ```

mod config;
mod context;
mod decisions;
mod error;
mod metrics;
mod oracle;

pub use config::OracleConfig;
pub use context::{CConstruct, CDecisionContext, LifetimeDecision};
pub use decisions::CDecisionCategory;
pub use error::OracleError;
pub use metrics::OracleMetrics;
pub use oracle::{DecyOracle, RustcError};

#[cfg(feature = "citl")]
pub use oracle::FixSuggestion;
