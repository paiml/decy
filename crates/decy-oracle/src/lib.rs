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

pub mod bootstrap;
mod config;
mod context;
pub mod dataset;
mod decisions;
pub mod diversity;
mod error;
pub mod golden_trace;
pub mod import;
mod metrics;
mod oracle;
pub mod retirement;
pub mod verification;

#[cfg(feature = "citl")]
pub use bootstrap::{create_bootstrapped_store, seed_pattern_store};
pub use bootstrap::{get_bootstrap_patterns, BootstrapPattern, BootstrapStats};
pub use config::OracleConfig;
pub use context::{CConstruct, CDecisionContext, LifetimeDecision};
pub use dataset::{
    generate_dataset_card, DatasetExporter, DatasetStats, ExportFormat, TrainingExample,
};
pub use decisions::CDecisionCategory;
pub use diversity::{
    analyze_corpus, categorize_error, compare_histograms, CConstruct as DiversityCConstruct,
    DiversityConfig, DiversityMetrics, DiversityValidation, ErrorCategory, ErrorHistogram,
};
pub use error::OracleError;
pub use import::{
    analyze_fix_strategy, smart_import_filter, FixStrategy, ImportDecision, ImportStats,
    SmartImportConfig, SourceLanguage,
};
pub use metrics::{CIReport, CIThresholds, OracleMetrics};
pub use oracle::{DecyOracle, RustcError};
pub use retirement::{
    run_retirement_sweep, PatternRetirementPolicy, PatternStats, RetirementConfig,
    RetirementDecision, RetirementReason, RetirementSweepResult,
};
pub use verification::{
    check_rust_compilation, run_test_suite, verify_fix_semantically, TestResult,
    VerificationConfig, VerificationResult, VerificationStats,
};

#[cfg(feature = "citl")]
pub use oracle::FixSuggestion;
