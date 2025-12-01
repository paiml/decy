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

    /// Export error
    #[error("Export error: {0}")]
    ExportError(String),
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

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // ERROR CONSTRUCTION TESTS
    // ============================================================================

    #[test]
    fn test_load_error_construction() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = OracleError::LoadError {
            path: "/tmp/test.apr".into(),
            source: io_err,
        };
        assert!(matches!(err, OracleError::LoadError { .. }));
    }

    #[test]
    fn test_save_error_construction() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err = OracleError::SaveError {
            path: "/etc/test.apr".into(),
            source: io_err,
        };
        assert!(matches!(err, OracleError::SaveError { .. }));
    }

    #[test]
    fn test_invalid_pattern_error() {
        let err = OracleError::InvalidPattern("malformed pattern syntax".into());
        assert!(matches!(err, OracleError::InvalidPattern(_)));
    }

    #[test]
    fn test_pattern_store_error() {
        let err = OracleError::PatternStoreError("store corruption detected".into());
        assert!(matches!(err, OracleError::PatternStoreError(_)));
    }

    #[test]
    fn test_config_error() {
        let err = OracleError::ConfigError("invalid threshold value".into());
        assert!(matches!(err, OracleError::ConfigError(_)));
    }

    #[test]
    fn test_diff_error() {
        let err = OracleError::DiffError("hunk mismatch at line 42".into());
        assert!(matches!(err, OracleError::DiffError(_)));
    }

    #[test]
    fn test_export_error() {
        let err = OracleError::ExportError("failed to write parquet".into());
        assert!(matches!(err, OracleError::ExportError(_)));
    }

    // ============================================================================
    // DISPLAY FORMATTING TESTS
    // ============================================================================

    #[test]
    fn test_load_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = OracleError::LoadError {
            path: "/tmp/patterns.apr".into(),
            source: io_err,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("/tmp/patterns.apr"));
        assert!(msg.contains("file not found"));
    }

    #[test]
    fn test_save_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err = OracleError::SaveError {
            path: "/etc/patterns.apr".into(),
            source: io_err,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("/etc/patterns.apr"));
        assert!(msg.contains("access denied"));
    }

    #[test]
    fn test_invalid_pattern_display() {
        let err = OracleError::InvalidPattern("missing error_code field".into());
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid pattern format"));
        assert!(msg.contains("missing error_code field"));
    }

    #[test]
    fn test_pattern_store_error_display() {
        let err = OracleError::PatternStoreError("index corruption".into());
        let msg = format!("{}", err);
        assert!(msg.contains("Pattern store error"));
        assert!(msg.contains("index corruption"));
    }

    #[test]
    fn test_config_error_display() {
        let err = OracleError::ConfigError("invalid TOML syntax".into());
        let msg = format!("{}", err);
        assert!(msg.contains("Configuration error"));
        assert!(msg.contains("invalid TOML syntax"));
    }

    #[test]
    fn test_diff_error_display() {
        let err = OracleError::DiffError("context mismatch".into());
        let msg = format!("{}", err);
        assert!(msg.contains("Failed to apply diff"));
        assert!(msg.contains("context mismatch"));
    }

    #[test]
    fn test_export_error_display() {
        let err = OracleError::ExportError("arrow schema error".into());
        let msg = format!("{}", err);
        assert!(msg.contains("Export error"));
        assert!(msg.contains("arrow schema error"));
    }

    // ============================================================================
    // FROM TRAIT IMPLEMENTATION TESTS
    // ============================================================================

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        let oracle_err: OracleError = io_err.into();

        match oracle_err {
            OracleError::LoadError { path, source } => {
                assert_eq!(path, "<unknown>");
                assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
            }
            _ => panic!("Expected LoadError variant"),
        }
    }

    #[test]
    fn test_from_io_error_preserves_error_kind() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let oracle_err: OracleError = io_err.into();

        if let OracleError::LoadError { source, .. } = oracle_err {
            assert_eq!(source.kind(), std::io::ErrorKind::PermissionDenied);
        } else {
            panic!("Expected LoadError variant");
        }
    }

    #[test]
    fn test_from_toml_error() {
        let toml_result: Result<toml::Value, _> = toml::from_str("invalid = [toml");
        let toml_err = toml_result.unwrap_err();
        let oracle_err: OracleError = toml_err.into();

        match oracle_err {
            OracleError::ConfigError(msg) => {
                assert!(!msg.is_empty());
            }
            _ => panic!("Expected ConfigError variant"),
        }
    }

    // ============================================================================
    // DEBUG FORMATTING TESTS
    // ============================================================================

    #[test]
    fn test_debug_formatting() {
        let err = OracleError::InvalidPattern("test".into());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("InvalidPattern"));
    }

    // ============================================================================
    // ERROR SOURCE CHAIN TESTS
    // ============================================================================

    #[test]
    fn test_load_error_source_chain() {
        use std::error::Error;

        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "underlying error");
        let err = OracleError::LoadError {
            path: "/test".into(),
            source: io_err,
        };

        // thiserror should provide source() method
        assert!(err.source().is_some());
    }

    #[test]
    fn test_save_error_source_chain() {
        use std::error::Error;

        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "disk full");
        let err = OracleError::SaveError {
            path: "/test".into(),
            source: io_err,
        };

        assert!(err.source().is_some());
    }

    #[test]
    fn test_simple_errors_no_source() {
        use std::error::Error;

        let err = OracleError::InvalidPattern("test".into());
        assert!(err.source().is_none());

        let err = OracleError::PatternStoreError("test".into());
        assert!(err.source().is_none());

        let err = OracleError::ConfigError("test".into());
        assert!(err.source().is_none());

        let err = OracleError::DiffError("test".into());
        assert!(err.source().is_none());

        let err = OracleError::ExportError("test".into());
        assert!(err.source().is_none());
    }
}
