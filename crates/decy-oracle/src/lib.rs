//! Decy Oracle: CITL pattern mining and fix suggestions for C-to-Rust transpilation
//!
//! This crate provides Tarantula fault localization to identify C language features
//! most correlated with transpilation failures, and suggest fixes.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────┐     ┌──────────────────┐
//! │  C Source Files     │────►│  CorpusCITL      │
//! │  (transpilation)    │     │  Pattern Mining  │
//! └─────────────────────┘     └──────────────────┘
//!                                      │
//!                                      ▼
//!                              ┌──────────────────┐
//!                              │  DecisionCITL    │
//!                              │  (Tarantula)     │
//!                              └──────────────────┘
//!                                      │
//!                                      ▼
//!                              ┌──────────────────┐
//!                              │  DecyOracle      │
//!                              │  (Fix Suggestions)│
//!                              └──────────────────┘
//! ```
//!
//! # Features
//!
//! - `citl`: Enable entrenar CITL integration for pattern mining

// TODO: Re-enable missing_docs when crate stabilizes
#![allow(missing_docs)]
#![deny(unsafe_code)]

// Core modules
pub mod baseline;
pub mod config;
pub mod context;
pub mod decisions;
pub mod error;
pub mod metrics;
pub mod oracle;

// Golden trace system
pub mod golden_trace;
pub mod trace_verifier;

// Dataset and import
pub mod bootstrap;
pub mod dataset;
pub mod diversity;
pub mod import;
pub mod retirement;
pub mod verification;

// CITL pattern mining (DECY-136)
#[cfg(feature = "citl")]
pub mod corpus_citl;

// Re-exports
pub use baseline::{aggregate_measurements, BaselineMetrics, FileMeasurement};
pub use config::OracleConfig;
pub use context::{CConstruct, CDecisionContext};
pub use decisions::CDecisionCategory;
pub use error::OracleError;
pub use golden_trace::{GoldenTrace, TraceTier};
pub use import::SmartImportConfig;
pub use metrics::{CIReport, CIThresholds, OracleMetrics};
pub use oracle::{DecyOracle, RustcError};
pub use retirement::PatternRetirementPolicy;
pub use trace_verifier::{TraceVerifier, VerificationLevel, VerifierConfig};

#[cfg(feature = "citl")]
pub use corpus_citl::{CFeature, CorpusCITL, IngestionStats};
