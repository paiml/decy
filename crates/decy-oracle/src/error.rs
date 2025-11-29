//! Oracle error types

use thiserror::Error;

/// Oracle error type
#[derive(Debug, Error)]
pub enum OracleError {
    /// Failed to load patterns file
    #[error("Failed to load patterns from {path}: {source}")]
    LoadError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to save patterns file
    #[error("Failed to save patterns to {path}: {source}")]
    SaveError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Invalid pattern format
    #[error("Invalid pattern format: {0}")]
    InvalidPattern(String),

    /// Pattern store error (from entrenar)
    #[error("Pattern store error: {0}")]
    PatternStoreError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Diff application failed
    #[error("Failed to apply diff: {0}")]
    DiffError(String),
}

impl From<std::io::Error> for OracleError {
    fn from(e: std::io::Error) -> Self {
        Self::LoadError {
            path: "<unknown>".into(),
            source: e,
        }
    }
}

impl From<toml::de::Error> for OracleError {
    fn from(e: toml::de::Error) -> Self {
        Self::ConfigError(e.to_string())
    }
}
